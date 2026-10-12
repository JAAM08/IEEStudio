//! Cálculo del proyecto completo: cada circuito, cada alimentador y el
//! cuadro de cargas de cada tablero.

use std::collections::HashMap;

use serde::Serialize;
use voltforge_core::{CircuitKind, Error, Size, System};
use voltforge_electrical::conduit::{size_conduit, ConductorGroup, ConduitInput, Covering};
use voltforge_electrical::current::{design_current, nominal_current};
use voltforge_electrical::sizing::{
    check_conductor, equipment_grounding, select_conductor, standard_rating_at_least, Governing,
    SizingInput, VoltageDropLimit,
};
use voltforge_electrical::voltage_drop::{voltage_drop, VoltageDropInput};

use crate::model::{LoadKind, Overrides, Panel, Phase, Project, Settings};
use crate::procedure::{
    conductor_steps, conduit_step, cumulative_step, current_step, drop_steps, ConductorData, Step,
};
use voltforge_rules::{Profile, Rule, RuleReference};

/// Resultado de una comprobación o de un elemento completo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Status {
    /// Se evaluó con todos los datos y está dentro del límite.
    Complies,
    /// Se evaluó y no cumple.
    Fails,
    /// Falta un dato o la regla aún no está implementada.
    NotEvaluated,
}

impl Status {
    /// Etiqueta en español para mostrar al usuario.
    pub fn label(self) -> &'static str {
        match self {
            Status::Complies => "Cumple",
            Status::Fails => "No cumple",
            Status::NotEvaluated => "No evaluado",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
    /// Dónde lo establece la norma.
    pub references: Vec<RuleReference>,
}

impl Check {
    /// Agrega las referencias normativas de las reglas que verifica.
    fn cite(mut self, profile: &Profile, rules: &[Rule]) -> Self {
        self.references = rules.iter().map(|&r| profile.reference(r)).collect();
        self
    }
}

fn check(name: &str, passes: bool, detail: String) -> Check {
    let status = if passes {
        Status::Complies
    } else {
        Status::Fails
    };
    Check {
        name: name.to_string(),
        status,
        detail,
        references: vec![],
    }
}

fn overall(checks: &[Check]) -> Status {
    if checks.iter().any(|c| c.status == Status::Fails) {
        Status::Fails
    } else if checks.iter().any(|c| c.status == Status::NotEvaluated) {
        Status::NotEvaluated
    } else {
        Status::Complies
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConduitChoice {
    pub conduit_type: String,
    pub metric_designator: u32,
    pub trade_size: String,
    pub fill_percent: f64,
    pub conductor_area_mm2: f64,
}

/// Valores calculados de un tramo: circuito derivado o alimentador.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Run {
    pub nominal_a: f64,
    pub design_a: f64,
    pub breaker_a: f64,
    pub poles: u32,
    pub length_m: f64,
    pub size: Size,
    /// El calibre lo fijó el usuario.
    pub forced: bool,
    /// Calibre que propone el programa. `None` si ninguno cumple.
    pub recommended_size: Option<Size>,
    /// Criterio que determinó el calibre propuesto.
    pub governing: Option<String>,
    pub conductors_per_phase: u32,
    /// Conductores de fase y neutro en el tramo, sin el de puesta a tierra.
    pub conductor_count: u32,
    pub current_carrying_conductors: u32,
    pub allowable_ampacity_a: f64,
    pub grounding_size: Size,
    pub conduit: Option<ConduitChoice>,
    pub drop_v: f64,
    pub drop_percent: f64,
    /// Caída acumulada desde la fuente hasta el final de este tramo.
    pub cumulative_drop_percent: f64,
    /// Procedimiento paso a paso de este tramo.
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunResult {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub load_w: f64,
    pub run: Option<Run>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelResult {
    pub id: String,
    pub name: String,
    /// Carga conectada, incluidos los tableros que alimenta.
    pub connected_w: f64,
    pub demand_factor: f64,
    pub demanded_w: f64,
    pub power_factor: f64,
    /// Carga conectada por fase, en VA: A, B y C.
    pub phase_va: [f64; 3],
    /// Desbalance entre fases, en porciento. `None` si no se asignaron fases.
    pub imbalance_percent: Option<f64>,
    pub feeder: RunResult,
    pub circuits: Vec<RunResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectResult {
    pub project: String,
    pub profile: String,
    pub panels: Vec<PanelResult>,
    pub complies: usize,
    pub fails: usize,
    pub not_evaluated: usize,
    /// Avisos que no cambian el estado de ningún elemento.
    pub warnings: Vec<String>,
}

struct RunSpec<'a> {
    kind: CircuitKind,
    power_w: f64,
    power_factor: f64,
    continuous_fraction: f64,
    length_m: f64,
    has_neutral: bool,
    overrides: &'a Overrides,
    drop_limit_percent: f64,
    /// Regla del límite de caída: derivado o alimentador.
    drop_rule: Rule,
    upstream_drop_percent: f64,
}

pub(crate) fn poles(kind: CircuitKind) -> u32 {
    match kind {
        CircuitKind::LineToNeutral => 1,
        CircuitKind::LineToLine => 2,
        CircuitKind::ThreePhase => 3,
    }
}

/// Un sistema en estrella tiene la tensión entre fases √3 veces la de fase a neutro.
pub(crate) fn is_wye(system: &System) -> bool {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(ll), Some(ln)) => (ll / ln - 3f64.sqrt()).abs() < 0.05,
        _ => false,
    }
}

fn governing_label(g: Governing) -> &'static str {
    match g {
        Governing::Ampacity => "ampacidad",
        Governing::Protection => "protección",
        Governing::VoltageDrop => "caída de tensión",
        Governing::UserMinimum => "mínimo del usuario",
    }
}

fn size_run(
    profile: &Profile,
    settings: &Settings,
    system: &System,
    spec: &RunSpec,
) -> Result<(Run, Vec<Check>), Error> {
    let ov = spec.overrides;
    let nominal_a = nominal_current(spec.power_w, spec.power_factor, spec.kind, system)?;
    let continuous_a = nominal_a * spec.continuous_fraction;
    let noncontinuous_a = nominal_a - continuous_a;
    let design_a = design_current(profile, continuous_a, noncontinuous_a);
    let n = ov.conductors_per_phase.unwrap_or(1);
    let material = ov.material.unwrap_or(settings.material);
    let insulation = ov.insulation.unwrap_or(settings.insulation);
    let conduit_type = ov.conduit.unwrap_or(settings.conduit);

    // Neutro como conductor portador de corriente, 310-15(b)(5).
    let has_neutral = spec.kind == CircuitKind::LineToNeutral || spec.has_neutral;
    let neutral_counts = has_neutral
        && match spec.kind {
            CircuitKind::LineToNeutral => true,
            CircuitKind::LineToLine => is_wye(system),
            CircuitKind::ThreePhase => ov.nonlinear,
        };
    let phase_conductors = poles(spec.kind) * n;
    let neutral_conductors = if has_neutral { n } else { 0 };
    let current_carrying_conductors = phase_conductors
        + if neutral_counts {
            neutral_conductors
        } else {
            0
        }
        + ov.other_current_carrying_conductors;

    let drop_input = VoltageDropInput {
        kind: spec.kind,
        system: *system,
        current_a: nominal_a,
        length_m: spec.length_m,
        power_factor: spec.power_factor,
        size: Size::new("14"),
        material,
        raceway: conduit_type.raceway_material(),
        conductors_per_phase: n,
        conductor_temperature_c: settings.resistance_temperature_c,
    };
    let sizing = SizingInput {
        continuous_a,
        noncontinuous_a,
        material,
        grounding_material: settings.material,
        insulation,
        wet_location: settings.wet_location,
        ambient_c: settings.ambient_c,
        current_carrying_conductors,
        nipple: ov.nipple,
        conductors_per_phase: n,
        terminal_rating: None,
        include_iec_ratings: settings.include_iec_ratings,
        minimum_size: None,
        voltage_drop: Some(VoltageDropLimit {
            input: drop_input.clone(),
            limit_percent: spec.drop_limit_percent,
        }),
    };

    let breaker_a = standard_rating_at_least(profile, design_a, settings.include_iec_ratings)?;
    let recommended = select_conductor(profile, &sizing);
    let (conductor, grounding_size) = match (&ov.size, &recommended) {
        (Some(forced), _) => {
            let conductor = check_conductor(profile, &sizing, forced, breaker_a)?;
            let base = recommended.as_ref().map_or(forced, |r| &r.ampacity_size);
            let grounding = equipment_grounding(
                profile,
                breaker_a,
                settings.material,
                settings.material == material,
                base,
                forced,
            )?;
            (conductor, grounding.size)
        }
        (None, Ok(r)) => (r.conductor.clone(), r.grounding.size.clone()),
        (None, Err(e)) => return Err(e.clone()),
    };

    let final_drop_input = VoltageDropInput {
        size: conductor.size.clone(),
        ..drop_input
    };
    let drop = voltage_drop(profile, &final_drop_input)?;
    let cumulative_drop_percent = spec.upstream_drop_percent + drop.drop_percent;

    let mut steps = vec![current_step(
        spec.kind,
        Some(spec.power_w),
        spec.power_factor,
        drop.reference_voltage_v,
        nominal_a,
    )];
    steps.extend(conductor_steps(&ConductorData {
        profile,
        continuous_a,
        noncontinuous_a,
        design_a,
        breaker_a,
        poles: poles(spec.kind),
        material,
        insulation,
        ambient_c: settings.ambient_c,
        current_carrying_conductors,
        check: &conductor,
        minimum_size: recommended.as_ref().ok().map(|r| &r.ampacity_size),
        grounding: Some((&grounding_size, settings.material)),
    }));

    let mut checks = vec![
        check(
            "Ampacidad",
            conductor.meets_ampacity(),
            format!(
                "{:.2} A permitidos para {:.2} A de carga y {:.2} A de diseño",
                conductor.allowable_ampacity_a, nominal_a, design_a
            ),
        )
        .cite(
            profile,
            &[
                Rule::DesignCurrent,
                Rule::Ampacity,
                Rule::TemperatureCorrection,
                Rule::Bundling,
                Rule::Terminals,
            ],
        ),
        check(
            "Protección",
            conductor.protected,
            format!(
                "interruptor de {breaker_a:.0} A con conductor de {:.2} A",
                conductor.allowable_ampacity_a
            ),
        )
        .cite(profile, &[Rule::ConductorProtection, Rule::SmallConductors]),
        check(
            "Caída de tensión",
            drop.drop_percent <= spec.drop_limit_percent,
            format!(
                "{:.2} % de {:.2} % permitido",
                drop.drop_percent, spec.drop_limit_percent
            ),
        )
        .cite(profile, &[spec.drop_rule]),
        check(
            "Caída acumulada",
            cumulative_drop_percent <= settings.total_drop_limit_percent,
            format!(
                "{cumulative_drop_percent:.2} % de {:.2} % permitido",
                settings.total_drop_limit_percent
            ),
        )
        .cite(profile, &[Rule::BranchDrop, Rule::FeederDrop]),
    ];
    if n > 1 && !conductor.parallel_allowed {
        checks.push(
            check(
                "Conductores en paralelo",
                false,
                format!(
                    "solo se permiten desde {}",
                    profile.parameters.smallest_parallel_size
                ),
            )
            .cite(profile, &[Rule::Parallel]),
        );
    }

    let dims = insulation.dimension_group();
    let mut groups = vec![ConductorGroup {
        count: phase_conductors + neutral_conductors,
        size: conductor.size.clone(),
        covering: Covering::Insulated(dims),
    }];
    groups.push(ConductorGroup {
        count: 1,
        size: grounding_size.clone(),
        covering: if settings.bare_grounding {
            Covering::Bare
        } else {
            Covering::Insulated(dims)
        },
    });
    let conduit = match size_conduit(
        profile,
        &ConduitInput {
            groups,
            conduit: conduit_type,
            nipple: ov.nipple,
        },
    ) {
        Ok(c) => {
            steps.push(conduit_step(profile, &c, conduit_type, ov.nipple));
            checks.push(
                check(
                    "Tubería",
                    true,
                    format!(
                        "{conduit_type} de {} mm, ocupación de {:.1} %",
                        c.conduit.metric_designator, c.fill_percent
                    ),
                )
                .cite(profile, &[Rule::ConduitFill, Rule::ConduitDimensions]),
            );
            Some(ConduitChoice {
                conduit_type: conduit_type.to_string(),
                metric_designator: c.conduit.metric_designator,
                trade_size: c.conduit.trade_size.clone(),
                fill_percent: c.fill_percent,
                conductor_area_mm2: c.conductor_area_mm2,
            })
        }
        Err(e @ Error::NoConduitFits { .. }) => {
            checks.push(check("Tubería", false, e.to_string()));
            None
        }
        Err(e) => {
            checks.push(Check {
                name: "Tubería".to_string(),
                status: Status::NotEvaluated,
                detail: e.to_string(),
                references: vec![],
            });
            None
        }
    };

    steps.extend(drop_steps(
        profile,
        &final_drop_input,
        &drop,
        spec.drop_limit_percent,
        Some(spec.drop_rule),
    ));
    steps.push(cumulative_step(
        profile,
        spec.upstream_drop_percent,
        drop.drop_percent,
        settings.total_drop_limit_percent,
    ));

    let run = Run {
        nominal_a,
        design_a,
        breaker_a,
        poles: poles(spec.kind),
        length_m: spec.length_m,
        size: conductor.size.clone(),
        forced: ov.size.is_some(),
        recommended_size: recommended.as_ref().ok().map(|r| r.conductor.size.clone()),
        governing: recommended
            .as_ref()
            .ok()
            .map(|r| governing_label(r.governing).to_string()),
        conductors_per_phase: n,
        conductor_count: phase_conductors + neutral_conductors,
        current_carrying_conductors,
        allowable_ampacity_a: conductor.allowable_ampacity_a,
        grounding_size,
        conduit,
        drop_v: drop.drop_v,
        drop_percent: drop.drop_percent,
        cumulative_drop_percent,
        steps,
    };
    Ok((run, checks))
}

fn run_result(
    id: &str,
    name: &str,
    load_w: f64,
    outcome: Result<(Run, Vec<Check>), Error>,
    extra: Vec<Check>,
) -> RunResult {
    let (run, mut checks) = match outcome {
        Ok((run, checks)) => (Some(run), checks),
        Err(e) => (
            None,
            vec![Check {
                name: "Cálculo".to_string(),
                status: Status::NotEvaluated,
                detail: e.to_string(),
                references: vec![],
            }],
        ),
    };
    checks.extend(extra);
    RunResult {
        id: id.to_string(),
        name: name.to_string(),
        status: overall(&checks),
        load_w,
        run,
        checks,
    }
}

/// Carga agregada de un tablero, incluidos sus subtableros.
#[derive(Debug, Clone, Copy, Default)]
struct PanelLoad {
    connected_w: f64,
    continuous_w: f64,
    apparent_va: f64,
    phase_va: [f64; 3],
    phases_assigned: bool,
}

struct Context<'a> {
    project: &'a Project,
    profile: &'a Profile,
    children: HashMap<&'a str, Vec<usize>>,
    loads: Vec<PanelLoad>,
    results: Vec<Option<PanelResult>>,
    warnings: Vec<String>,
}

fn validate(project: &Project) -> Result<(), String> {
    let mut seen = HashMap::new();
    for (i, p) in project.panels.iter().enumerate() {
        if seen.insert(p.id.as_str(), i).is_some() {
            return Err(format!("hay dos tableros con el identificador «{}»", p.id));
        }
        let mut ids = std::collections::HashSet::new();
        for c in &p.circuits {
            if !ids.insert(c.id.as_str()) {
                return Err(format!(
                    "el tablero «{}» repite el circuito «{}»",
                    p.id, c.id
                ));
            }
        }
    }
    for p in &project.panels {
        if let Some(parent) = &p.parent {
            if !seen.contains_key(parent.as_str()) {
                return Err(format!(
                    "el tablero «{}» se alimenta de «{parent}», que no existe",
                    p.id
                ));
            }
        }
        // Subir por la cadena de alimentación no debe regresar al mismo tablero.
        let mut at = p;
        for _ in 0..=project.panels.len() {
            match &at.parent {
                None => break,
                Some(parent) if parent == &p.id => {
                    return Err(format!("el tablero «{}» se alimenta de sí mismo", p.id));
                }
                Some(parent) => at = &project.panels[seen[parent.as_str()]],
            }
        }
    }
    Ok(())
}

impl<'a> Context<'a> {
    fn aggregate(&mut self, index: usize) -> PanelLoad {
        let panel = &self.project.panels[index];
        let mut load = PanelLoad {
            phases_assigned: true,
            ..PanelLoad::default()
        };
        for c in &panel.circuits {
            let pf = c.load.power_factor;
            let va = if pf > 0.0 { c.load.power_w / pf } else { 0.0 };
            load.connected_w += c.load.power_w;
            load.apparent_va += va;
            if c.load.continuous {
                load.continuous_w += c.load.power_w;
            }
            let expected = poles(c.kind) as usize;
            if c.phases.len() == expected {
                for phase in &c.phases {
                    let slot = match phase {
                        Phase::A => 0,
                        Phase::B => 1,
                        Phase::C => 2,
                    };
                    load.phase_va[slot] += va / expected as f64;
                }
            } else {
                load.phases_assigned = false;
            }
        }
        for child in self
            .children
            .get(panel.id.as_str())
            .cloned()
            .unwrap_or_default()
        {
            let sub = self.aggregate(child);
            load.connected_w += sub.connected_w;
            load.continuous_w += sub.continuous_w;
            load.apparent_va += sub.apparent_va;
            for i in 0..3 {
                load.phase_va[i] += sub.phase_va[i];
            }
            load.phases_assigned &= sub.phases_assigned;
        }
        self.loads[index] = load;
        load
    }

    fn compute(&mut self, index: usize, upstream_drop_percent: f64) {
        let project = self.project;
        let panel: &Panel = &project.panels[index];
        let settings = &project.settings;
        let system = &project.source.system;
        let load = self.loads[index];

        let power_factor = if load.apparent_va > 0.0 {
            load.connected_w / load.apparent_va
        } else {
            1.0
        };
        let demanded_w = load.connected_w * panel.feeder.demand_factor;
        let feeder = if load.connected_w > 0.0 {
            let outcome = size_run(
                self.profile,
                settings,
                system,
                &RunSpec {
                    kind: panel.kind,
                    power_w: demanded_w,
                    power_factor,
                    continuous_fraction: load.continuous_w / load.connected_w,
                    length_m: panel.feeder.length_m,
                    has_neutral: panel.feeder.has_neutral,
                    overrides: &panel.feeder.overrides,
                    drop_limit_percent: settings.feeder_drop_limit_percent,
                    drop_rule: Rule::FeederDrop,
                    upstream_drop_percent,
                },
            );
            run_result(&panel.id, "Alimentador", demanded_w, outcome, vec![])
        } else {
            run_result(
                &panel.id,
                "Alimentador",
                0.0,
                Err(Error::InvalidInput("el tablero no tiene carga".into())),
                vec![],
            )
        };
        let panel_drop_percent = feeder
            .run
            .as_ref()
            .map_or(upstream_drop_percent, |r| r.cumulative_drop_percent);

        let circuits = panel
            .circuits
            .iter()
            .map(|c| {
                let outcome = size_run(
                    self.profile,
                    settings,
                    system,
                    &RunSpec {
                        kind: c.kind,
                        power_w: c.load.power_w,
                        power_factor: c.load.power_factor,
                        continuous_fraction: if c.load.continuous { 1.0 } else { 0.0 },
                        length_m: c.length_m,
                        has_neutral: c.has_neutral,
                        overrides: &c.overrides,
                        drop_limit_percent: settings.branch_drop_limit_percent,
                        drop_rule: Rule::BranchDrop,
                        upstream_drop_percent: panel_drop_percent,
                    },
                );
                let pending = |rule: &str| Check {
                    name: rule.to_string(),
                    status: Status::NotEvaluated,
                    detail: "calculado como carga general; sus reglas propias aún no se aplican"
                        .to_string(),
                    references: vec![],
                };
                let extra = match c.load.kind {
                    LoadKind::Motor => vec![pending("Reglas de motores")],
                    LoadKind::AirConditioning => vec![pending("Reglas de climatización")],
                    LoadKind::General | LoadKind::Lighting => vec![],
                };
                run_result(&c.id, &c.name, c.load.power_w, outcome, extra)
            })
            .collect();

        let imbalance_percent = (load.phases_assigned && load.connected_w > 0.0).then(|| {
            let mut used = load.phase_va;
            used.sort_by(|a, b| b.total_cmp(a));
            let count = poles(panel.kind) as usize;
            let (max, min) = (used[0], used[count - 1]);
            if max > 0.0 {
                (max - min) / max * 100.0
            } else {
                0.0
            }
        });
        if let Some(imbalance) = imbalance_percent {
            if imbalance > settings.imbalance_warning_percent {
                self.warnings.push(format!(
                    "{}: desbalance entre fases de {imbalance:.1} %",
                    panel.name
                ));
            }
        }

        self.results[index] = Some(PanelResult {
            id: panel.id.clone(),
            name: panel.name.clone(),
            connected_w: load.connected_w,
            demand_factor: panel.feeder.demand_factor,
            demanded_w,
            power_factor,
            phase_va: load.phase_va,
            imbalance_percent,
            feeder,
            circuits,
        });
        for child in self
            .children
            .get(panel.id.as_str())
            .cloned()
            .unwrap_or_default()
        {
            self.compute(child, panel_drop_percent);
        }
    }
}

/// Calcula todo el proyecto.
///
/// Devuelve `Err` solo si la estructura del proyecto es inválida. Un dato
/// faltante o un caso que las tablas no cubren no es un error: el elemento
/// queda como «no evaluado» con su explicación.
pub fn calculate(project: &Project) -> Result<ProjectResult, String> {
    let profile = Profile::by_id(&project.profile)
        .ok_or_else(|| format!("perfil normativo desconocido: «{}»", project.profile))?;
    validate(project)?;

    let mut children: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();
    for (i, p) in project.panels.iter().enumerate() {
        match &p.parent {
            Some(parent) => children.entry(parent.as_str()).or_default().push(i),
            None => roots.push(i),
        }
    }
    let count = project.panels.len();
    let mut ctx = Context {
        project,
        profile,
        children,
        loads: vec![PanelLoad::default(); count],
        results: vec![None; count],
        warnings: Vec::new(),
    };
    for &root in &roots {
        ctx.aggregate(root);
        ctx.compute(root, 0.0);
    }

    let panels: Vec<PanelResult> = ctx.results.into_iter().flatten().collect();
    let statuses: Vec<Status> = panels
        .iter()
        .flat_map(|p| std::iter::once(p.feeder.status).chain(p.circuits.iter().map(|c| c.status)))
        .collect();
    let tally = |s: Status| statuses.iter().filter(|&&x| x == s).count();
    Ok(ProjectResult {
        project: project.name.clone(),
        profile: profile.title.to_string(),
        complies: tally(Status::Complies),
        fails: tally(Status::Fails),
        not_evaluated: tally(Status::NotEvaluated),
        panels,
        warnings: ctx.warnings,
    })
}
