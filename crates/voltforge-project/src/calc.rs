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

use crate::demand::{feeder_demand, outlet_check, DemandLine, Groups};
use crate::fault::{
    bus_step, interrupting_check, missing_source, source_fault, unknown, SourceFault, Upstream,
};
use crate::model::{Overrides, Panel, Phase, Project, Settings};
use crate::motors::{motor_circuit, MotorCircuit};
use crate::procedure::{
    conductor_steps, conduit_step, cumulative_step, current_step, drop_steps, ConductorData, Step,
};
use crate::verdict::Verdict;
use voltforge_electrical::short_circuit::{conductor_impedance, fault_current_a};
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

pub(crate) fn check(name: &str, passes: bool, detail: String) -> Check {
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
    /// El interruptor lo fijó el usuario.
    pub breaker_forced: bool,
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
    /// Corriente de cortocircuito disponible en las terminales de línea de su
    /// interruptor, en kA. `None` si no se calculó.
    pub fault_ka: Option<f64>,
    /// Capacidad interruptiva del interruptor, en kA, si se capturó.
    pub interrupting_ka: Option<f64>,
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
    /// Factor de demanda resultante: carga calculada entre conectada.
    pub demand_factor: f64,
    pub demanded_w: f64,
    /// Carga calculada del alimentador, en VA (artículo 220).
    pub demanded_va: f64,
    /// Renglones de la carga calculada: grupo, conectada, factor y calculada.
    pub demand: Vec<DemandLine>,
    /// Corriente de cortocircuito en las barras del tablero, en kA.
    pub fault_ka: Option<f64>,
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
    /// Corriente de cortocircuito de la fuente, en kA, con la aportación de
    /// los motores.
    pub source_fault_ka: Option<f64>,
    /// Aportación de los motores a la falla, en kA.
    pub motor_fault_ka: Option<f64>,
    /// Cálculo, verificación normativa y aprobación profesional, por separado.
    pub verdict: Verdict,
}

/// Corriente de un tramo que ya resolvió quien llama: circuitos de motor o de
/// compresor y alimentadores con motores.
pub(crate) struct Given {
    pub nominal_a: f64,
    pub design_a: f64,
    /// Interruptor propuesto. `None` toma el normalizado igual o inmediato
    /// superior a la corriente de diseño.
    pub breaker_a: Option<f64>,
    /// Circuito derivado de motor: el conductor se protege según el artículo 430.
    pub motor_protection: bool,
    /// Pasos de corriente, diseño e interruptor.
    pub steps: Vec<Step>,
    pub ampacity_rules: Vec<Rule>,
    /// Alimentador con motores: interruptor máximo según 430-62.
    pub feeder_motor_limit_a: Option<f64>,
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
    given: Option<Given>,
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

/// √3 en tres fases; 1 en una o dos fases.
pub(crate) fn phase_multiplier(kind: CircuitKind) -> f64 {
    match kind {
        CircuitKind::ThreePhase => voltforge_core::units::SQRT_3,
        CircuitKind::LineToNeutral | CircuitKind::LineToLine => 1.0,
    }
}

/// Alimentador con motores (430-24, 440-33): 125 % de la carga continua que
/// no es motor, más el resto, más el 25 % del motor mayor. Su interruptor
/// puede llegar al del motor mayor más el resto de la carga (430-62, 430-63).
fn feeder_with_motors(
    profile: &Profile,
    system: &System,
    kind: CircuitKind,
    demanded_w: f64,
    power_factor: f64,
    continuous_fraction: f64,
    largest: &LargestMotor,
) -> Result<Given, Error> {
    let p = &profile.parameters;
    let nominal_a = nominal_current(demanded_w, power_factor, kind, system)?;
    let volts = kind.reference_voltage(system)?;
    let k = phase_multiplier(kind);
    let continuous_a = nominal_a * continuous_fraction;
    let noncontinuous_a = nominal_a - continuous_a;
    let motor_a = largest.va / (k * volts);
    let extra = p.motor_largest_factor - 1.0;
    let design_a = p.continuous_load_factor * continuous_a + noncontinuous_a + extra * motor_a;
    let limit_a = nominal_a + (largest.breaker_va - largest.va) / (k * volts);
    let r = |rules: &[Rule]| rules.iter().map(|&x| profile.reference(x)).collect();
    let steps = vec![
        current_step(kind, Some(demanded_w), power_factor, volts, nominal_a),
        Step {
            title: "Corriente de diseño".into(),
            formula: Some("Id = 1.25 · I continua + I no continua + 0.25 · I motor mayor".into()),
            substitution: format!(
                "Id = {} · {:.2} + {:.2} + {} · {:.2} = {:.2} A",
                crate::procedure::plain(p.continuous_load_factor),
                continuous_a,
                noncontinuous_a,
                crate::procedure::plain(extra),
                motor_a,
                design_a
            ),
            note: Some(format!(
                "Motor mayor: {}{}. Su corriente se lleva a la tensión del alimentador por su potencia aparente. Los motores cuentan al 100 % en la carga y el mayor suma otro 25 %.",
                largest.label,
                if largest.panel.is_empty() { String::new() } else { format!(", tablero {}", largest.panel) }
            )),
            references: r(&[Rule::MotorFeederConductor, Rule::CompressorConductor, Rule::DesignCurrent]),
        },
    ];
    Ok(Given {
        nominal_a,
        design_a,
        breaker_a: None,
        motor_protection: false,
        steps,
        ampacity_rules: vec![Rule::MotorFeederConductor, Rule::DesignCurrent],
        feeder_motor_limit_a: Some(limit_a),
    })
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
    let given = spec.given.as_ref();
    let (nominal_a, continuous_a, noncontinuous_a, design_a) = match given {
        Some(g) => (g.nominal_a, 0.0, g.nominal_a, g.design_a),
        None => {
            let nominal = nominal_current(spec.power_w, spec.power_factor, spec.kind, system)?;
            let continuous = nominal * spec.continuous_fraction;
            let noncontinuous = nominal - continuous;
            (
                nominal,
                continuous,
                noncontinuous,
                design_current(profile, continuous, noncontinuous),
            )
        }
    };
    let motor_protection = given.is_some_and(|g| g.motor_protection);
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
        design_a: given.map(|g| g.design_a),
        breaker_a: None,
        motor_protection,
    };

    let breaker_a = match ov.breaker_a.or(given.and_then(|g| g.breaker_a)) {
        Some(b) => b,
        None => standard_rating_at_least(profile, design_a, settings.include_iec_ratings)?,
    };
    let sizing = SizingInput {
        breaker_a: Some(breaker_a),
        ..sizing
    };
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

    let mut steps = match given {
        Some(g) => g.steps.clone(),
        None => vec![current_step(
            spec.kind,
            Some(spec.power_w),
            spec.power_factor,
            drop.reference_voltage_v,
            nominal_a,
        )],
    };
    steps.extend(conductor_steps(&ConductorData {
        profile,
        continuous_a,
        noncontinuous_a,
        design_a,
        breaker_a,
        breaker_forced: ov.breaker_a.is_some(),
        given_design: given.is_some(),
        motor_protection,
        poles: poles(spec.kind),
        material,
        insulation,
        ambient_c: settings.ambient_c,
        current_carrying_conductors,
        check: &conductor,
        minimum_size: recommended.as_ref().ok().map(|r| &r.ampacity_size),
        grounding: Some((&grounding_size, settings.material)),
    }));

    let mut ampacity_rules = given.map_or(vec![Rule::DesignCurrent], |g| g.ampacity_rules.clone());
    ampacity_rules.extend([
        Rule::Ampacity,
        Rule::TemperatureCorrection,
        Rule::Bundling,
        Rule::Terminals,
    ]);
    let mut checks = vec![check(
        "Ampacidad",
        conductor.meets_ampacity(),
        format!(
            "{:.2} A permitidos para {:.2} A de carga y {:.2} A de diseño",
            conductor.allowable_ampacity_a, nominal_a, design_a
        ),
    )
    .cite(profile, &ampacity_rules)];
    // En un circuito de motor la protección la verifica su propia regla.
    if !motor_protection {
        let motor_limit = given.and_then(|g| g.feeder_motor_limit_a);
        let by_430_62 = motor_limit.is_some_and(|max| breaker_a <= max + 1e-9);
        let covers_design = breaker_a >= design_a - 1e-9;
        let protected = conductor.protected || by_430_62;
        let detail = if !covers_design {
            format!("el interruptor de {breaker_a:.0} A es menor que la corriente de diseño de {design_a:.2} A")
        } else if conductor.protected {
            format!(
                "interruptor de {breaker_a:.0} A con conductor de {:.2} A",
                conductor.allowable_ampacity_a
            )
        } else if by_430_62 {
            format!(
                "interruptor de {breaker_a:.0} A: mayor que la ampacidad, dentro de los {:.0} A que permite 430-62 para alimentadores de motores",
                motor_limit.unwrap_or_default()
            )
        } else {
            format!(
                "interruptor de {breaker_a:.0} A con conductor de {:.2} A",
                conductor.allowable_ampacity_a
            )
        };
        let mut rules = vec![Rule::ConductorProtection, Rule::SmallConductors];
        if ov.breaker_a.is_some() {
            rules.push(Rule::BreakerRating);
        }
        if motor_limit.is_some() {
            rules.push(Rule::MotorFeederProtection);
        }
        checks.push(check("Protección", protected && covers_design, detail).cite(profile, &rules));
    }
    checks.extend([
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
    ]);
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
        breaker_forced: ov.breaker_a.is_some(),
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
        fault_ka: None,
        interrupting_ka: None,
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

/// Motor o compresor de mayor potencia aguas abajo de un tablero.
#[derive(Debug, Clone, Default)]
struct LargestMotor {
    va: f64,
    /// Potencia equivalente a su interruptor máximo, para 430-62.
    breaker_va: f64,
    label: String,
    panel: String,
}

/// Carga agregada de un tablero, incluidos sus subtableros.
#[derive(Debug, Clone, Default)]
struct PanelLoad {
    connected_w: f64,
    apparent_va: f64,
    /// Carga por grupo del artículo 220, en VA.
    groups: Groups,
    phase_va: [f64; 3],
    phases_assigned: bool,
    largest_motor: Option<LargestMotor>,
}

/// Potencia de un circuito: la de sus datos de motor o compresor si los
/// tiene, si no la capturada.
struct CircuitLoad {
    power_w: f64,
    va: f64,
}

struct Context<'a> {
    project: &'a Project,
    profile: &'a Profile,
    children: HashMap<&'a str, Vec<usize>>,
    /// Datos de motor o compresor de cada circuito, por tablero.
    motors: Vec<Vec<Option<Result<MotorCircuit, Error>>>>,
    loads: Vec<PanelLoad>,
    results: Vec<Option<PanelResult>>,
    warnings: Vec<String>,
    source_fault: Option<Result<SourceFault, String>>,
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
    fn circuit_load(&self, panel: usize, circuit: usize) -> CircuitLoad {
        let c = &self.project.panels[panel].circuits[circuit];
        let pf = c.load.power_factor;
        match &self.motors[panel][circuit] {
            Some(Ok(m)) => CircuitLoad {
                power_w: m.va * pf,
                va: m.va,
            },
            _ => CircuitLoad {
                power_w: c.load.power_w,
                va: if pf > 0.0 { c.load.power_w / pf } else { 0.0 },
            },
        }
    }

    fn aggregate(&mut self, index: usize) -> PanelLoad {
        let panel = &self.project.panels[index];
        let mut load = PanelLoad {
            phases_assigned: true,
            ..PanelLoad::default()
        };
        load.groups.areas = panel.areas.clone();
        for (k, c) in panel.circuits.iter().enumerate() {
            let CircuitLoad { power_w, va } = self.circuit_load(index, k);
            load.connected_w += power_w;
            load.apparent_va += va;
            let is_motor = matches!(self.motors[index][k], Some(Ok(_)));
            load.groups
                .add(c.load.kind, is_motor, va, c.load.continuous);
            if let Some(Ok(m)) = &self.motors[index][k] {
                let candidate = LargestMotor {
                    va: m.va,
                    breaker_va: m.va * m.breaker_max_a / m.current_a,
                    label: format!("{} ({})", m.label, c.id),
                    panel: panel.id.clone(),
                };
                if load
                    .largest_motor
                    .as_ref()
                    .map_or(true, |l| candidate.va > l.va)
                {
                    load.largest_motor = Some(candidate);
                }
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
            load.apparent_va += sub.apparent_va;
            load.groups.merge(&sub.groups);
            for i in 0..3 {
                load.phase_va[i] += sub.phase_va[i];
            }
            load.phases_assigned &= sub.phases_assigned;
            if let Some(m) = sub.largest_motor {
                if load.largest_motor.as_ref().map_or(true, |l| m.va > l.va) {
                    load.largest_motor = Some(m);
                }
            }
        }
        self.loads[index] = load.clone();
        load
    }

    fn compute(&mut self, index: usize, upstream_drop_percent: f64, upstream: Upstream) {
        let project = self.project;
        let panel: &Panel = &project.panels[index];
        let settings = &project.settings;
        let system = &project.source.system;
        let load = self.loads[index].clone();

        let power_factor = if load.apparent_va > 0.0 {
            load.connected_w / load.apparent_va
        } else {
            1.0
        };
        let demand = feeder_demand(
            self.profile,
            project.occupancy.as_deref(),
            panel.feeder.demand,
            &load.groups,
        );
        // La corriente sale de la potencia aparente calculada; los watts se
        // obtienen con el factor de potencia de la carga conectada.
        let demanded_w = demand.demanded_va * power_factor;
        let continuous_fraction = if demand.demanded_va > 0.0 {
            (demand.continuous_va / demand.demanded_va).min(1.0)
        } else {
            0.0
        };
        let given = match &load.largest_motor {
            Some(m) if load.connected_w > 0.0 => feeder_with_motors(
                self.profile,
                system,
                panel.kind,
                demanded_w,
                power_factor,
                continuous_fraction,
                m,
            )
            .ok(),
            _ => None,
        };
        let motor_breaker_a = match (&load.largest_motor, panel.kind.reference_voltage(system)) {
            (Some(m), Ok(v)) => {
                Some(m.breaker_va / (v * crate::calc::phase_multiplier(panel.kind)))
            }
            _ => None,
        };
        // Capacidad interruptiva del interruptor del alimentador, con la falla
        // del punto de donde sale.
        let feeder_fault_check = match &upstream {
            Upstream::NoData => panel.parent.is_none().then(|| missing_source(self.profile)),
            Upstream::Unknown(reason) => {
                Some(interrupting_check(self.profile, Err(reason.clone()), None))
            }
            Upstream::Known {
                fault_a,
                rating_ka,
                place,
                ..
            } => Some(interrupting_check(
                self.profile,
                Ok((*fault_a, place)),
                panel.feeder.interrupting_ka.or(*rating_ka),
            )),
        };
        let source = self.source_fault.as_ref().and_then(|s| s.as_ref().ok());
        let mut bus = match &upstream {
            Upstream::Known { .. } => {
                Upstream::Unknown("no se calculó el alimentador del tablero".into())
            }
            other => other.clone(),
        };
        let feeder = if load.connected_w > 0.0 {
            let mut outcome = size_run(
                self.profile,
                settings,
                system,
                &RunSpec {
                    kind: panel.kind,
                    power_w: demanded_w,
                    power_factor,
                    continuous_fraction,
                    length_m: panel.feeder.length_m,
                    has_neutral: panel.feeder.has_neutral,
                    overrides: &panel.feeder.overrides,
                    drop_limit_percent: settings.feeder_drop_limit_percent,
                    drop_rule: Rule::FeederDrop,
                    upstream_drop_percent,
                    given,
                },
            );
            // Coordinación: un interruptor de alimentador menor que el del
            // motor mayor puede dispararse cuando el motor arranca.
            if let (Ok((run, _)), Some(motor_a)) = (&outcome, motor_breaker_a) {
                if run.breaker_a + 1e-9 < motor_a {
                    self.warnings.push(format!(
                        "{}: el interruptor del alimentador ({:.0} A) es menor que el del motor mayor ({:.0} A); puede dispararse al arrancar el motor",
                        panel.name, run.breaker_a, motor_a
                    ));
                }
            }
            if let Ok((run, _)) = &mut outcome {
                run.steps.splice(0..0, demand.steps.iter().cloned());
                if let (
                    Upstream::Known {
                        impedance, fault_a, ..
                    },
                    Some(src),
                ) = (&upstream, source)
                {
                    run.fault_ka = Some(fault_a / 1000.0);
                    run.interrupting_ka = panel.feeder.interrupting_ka.or(match &upstream {
                        Upstream::Known { rating_ka, .. } => *rating_ka,
                        _ => None,
                    });
                    let ov = &panel.feeder.overrides;
                    let material = ov.material.unwrap_or(settings.material);
                    let raceway = ov.conduit.unwrap_or(settings.conduit).raceway_material();
                    bus = match conductor_impedance(
                        self.profile,
                        &run.size,
                        material,
                        raceway,
                        run.length_m,
                        run.conductors_per_phase,
                    ) {
                        Ok(zc) => {
                            let total = impedance.plus(zc.scaled(src.basis.loop_factor));
                            let bus_a = fault_current_a(&src.basis, total) + src.motor_a;
                            if panel.parent.is_none() {
                                run.steps.extend(src.steps.iter().cloned());
                            }
                            let description = format!(
                                "{} m de {}{}",
                                crate::procedure::plain(run.length_m),
                                if run.conductors_per_phase > 1 {
                                    format!("{} × ", run.conductors_per_phase)
                                } else {
                                    String::new()
                                },
                                crate::procedure::size_label(&run.size)
                            );
                            run.steps.push(bus_step(
                                self.profile,
                                src,
                                zc,
                                total,
                                bus_a,
                                &description,
                            ));
                            Upstream::Known {
                                impedance: total,
                                fault_a: bus_a,
                                rating_ka: panel.interrupting_ka,
                                place: panel.name.clone(),
                            }
                        }
                        Err(e) => unknown(&e),
                    };
                }
            }
            let mut extra = vec![demand.check.clone()];
            extra.extend(feeder_fault_check);
            run_result(&panel.id, "Alimentador", demanded_w, outcome, extra)
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
            .enumerate()
            .map(|(k, c)| {
                let motor = self.motors[index][k].clone();
                let given = match &motor {
                    Some(Ok(m)) => Some(Given {
                        nominal_a: m.current_a,
                        design_a: m.design_a,
                        breaker_a: Some(m.breaker_max_a),
                        motor_protection: true,
                        steps: m.steps.clone(),
                        ampacity_rules: m.ampacity_rules.clone(),
                        feeder_motor_limit_a: None,
                    }),
                    _ => None,
                };
                let mut outcome = size_run(
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
                        given,
                    },
                );
                let mut extra = match motor {
                    Some(Ok(m)) => m.checks,
                    Some(Err(e)) => vec![Check {
                        name: "Datos del equipo".to_string(),
                        status: Status::NotEvaluated,
                        detail: format!("{e}; se calculó como carga general"),
                        references: vec![],
                    }],
                    None => vec![],
                };
                extra.extend(outlet_check(self.profile, project.occupancy.as_deref(), c));
                match &bus {
                    Upstream::NoData => {}
                    Upstream::Unknown(reason) => {
                        extra.push(interrupting_check(self.profile, Err(reason.clone()), None))
                    }
                    Upstream::Known { fault_a, place, .. } => {
                        if let Ok((run, _)) = &mut outcome {
                            run.fault_ka = Some(fault_a / 1000.0);
                            run.interrupting_ka = panel.interrupting_ka;
                        }
                        extra.push(interrupting_check(
                            self.profile,
                            Ok((*fault_a, place)),
                            panel.interrupting_ka,
                        ));
                    }
                }
                let load_w = self.circuit_load(index, k).power_w;
                run_result(&c.id, &c.name, load_w, outcome, extra)
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
            demand_factor: if load.connected_w > 0.0 {
                demanded_w / load.connected_w
            } else {
                1.0
            },
            demanded_w,
            demanded_va: demand.demanded_va,
            demand: demand.lines,
            fault_ka: match &bus {
                Upstream::Known { fault_a, .. } => Some(fault_a / 1000.0),
                _ => None,
            },
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
            self.compute(child, panel_drop_percent, bus.clone());
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
    let motors: Vec<Vec<Option<Result<MotorCircuit, Error>>>> = project
        .panels
        .iter()
        .map(|p| {
            p.circuits
                .iter()
                .map(|c| motor_circuit(profile, &project.settings, &project.source.system, c))
                .collect()
        })
        .collect();
    let motors_va: f64 = motors
        .iter()
        .flatten()
        .filter_map(|m| m.as_ref()?.as_ref().ok())
        .map(|m| m.va)
        .sum();
    let source = source_fault(project, motors_va);
    let source_fault_ka = source
        .as_ref()
        .and_then(|s| s.as_ref().ok())
        .map(|s| (s.available_a + s.motor_a) / 1000.0);
    let motor_fault_ka = source
        .as_ref()
        .and_then(|s| s.as_ref().ok())
        .map(|s| s.motor_a / 1000.0);
    let root_upstream = match &source {
        None => Upstream::NoData,
        Some(Err(reason)) => Upstream::Unknown(reason.clone()),
        Some(Ok(s)) => Upstream::Known {
            impedance: s.impedance,
            fault_a: s.available_a + s.motor_a,
            rating_ka: None,
            place: project.source.name.clone(),
        },
    };
    let mut ctx = Context {
        project,
        profile,
        children,
        motors,
        loads: vec![PanelLoad::default(); count],
        results: vec![None; count],
        warnings: Vec::new(),
        source_fault: source,
    };
    for &root in &roots {
        ctx.aggregate(root);
        ctx.compute(root, 0.0, root_upstream.clone());
    }

    let panels: Vec<PanelResult> = ctx.results.into_iter().flatten().collect();
    let statuses: Vec<Status> = panels
        .iter()
        .flat_map(|p| std::iter::once(p.feeder.status).chain(p.circuits.iter().map(|c| c.status)))
        .collect();
    let tally = |s: Status| statuses.iter().filter(|&&x| x == s).count();
    let runs: Vec<&RunResult> = panels
        .iter()
        .flat_map(|p| std::iter::once(&p.feeder).chain(&p.circuits))
        .collect();
    let verdict = Verdict::new(
        runs.len(),
        runs.iter().filter(|r| r.run.is_none()).count(),
        // Las verificaciones de un tramo sin calcular ya cuentan como «sin calcular».
        runs.iter()
            .filter(|r| r.run.is_some())
            .flat_map(|r| r.checks.iter().map(|c| c.status)),
    );
    Ok(ProjectResult {
        project: project.name.clone(),
        profile: profile.title.to_string(),
        complies: tally(Status::Complies),
        fails: tally(Status::Fails),
        not_evaluated: tally(Status::NotEvaluated),
        panels,
        warnings: ctx.warnings,
        source_fault_ka,
        motor_fault_ka,
        verdict,
    })
}
