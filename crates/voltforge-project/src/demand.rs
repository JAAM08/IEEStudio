//! Carga de un alimentador según el artículo 220.
//!
//! La carga conectada de un tablero (sus circuitos y los de sus subtableros)
//! se separa en grupos: alumbrado general, contactos, aparatos fijos, equipo
//! de cocina, motores y otras cargas. A cada grupo se le aplica primero la
//! carga mínima que exige la norma (por área o por salida) y después el factor
//! de demanda que le permite su tipo de inmueble.

use serde::Serialize;
use voltforge_rules::{DemandTier, Occupancy, Profile, Rule, RuleReference};

use crate::calc::{Check, Status};
use crate::model::{Area, Circuit, Demand, LoadKind};
use crate::procedure::{plain, Step};

/// Un renglón de la carga del alimentador.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DemandLine {
    pub label: String,
    /// Suma de lo capturado en los circuitos, en VA.
    pub connected_va: f64,
    /// Carga sobre la que se aplica el factor: la capturada o la mínima que
    /// exige la norma, la mayor.
    pub basis_va: f64,
    pub demanded_va: f64,
    /// Cómo se obtuvo, con los valores: `3,000 × 100 % + 2,100 × 35 %`.
    pub detail: String,
    pub references: Vec<RuleReference>,
}

/// Carga de un grupo, en VA.
#[derive(Debug, Clone, Default)]
pub(crate) struct Part {
    pub va: f64,
    pub continuous_va: f64,
    /// Potencia de cada circuito del grupo: para contar aparatos y equipos.
    pub items: Vec<f64>,
}

impl Part {
    fn add(&mut self, va: f64, continuous: bool) {
        self.va += va;
        if continuous {
            self.continuous_va += va;
        }
        self.items.push(va);
    }

    fn merge(&mut self, other: &Part) {
        self.va += other.va;
        self.continuous_va += other.continuous_va;
        self.items.extend_from_slice(&other.items);
    }

    /// Fracción continua de lo capturado.
    fn continuous_fraction(&self) -> f64 {
        if self.va > 0.0 {
            self.continuous_va / self.va
        } else {
            0.0
        }
    }
}

/// Carga conectada aguas abajo de un tablero, por grupo del artículo 220.
#[derive(Debug, Clone, Default)]
pub(crate) struct Groups {
    pub lighting: Part,
    pub receptacles: Part,
    pub appliances: Part,
    pub kitchen: Part,
    pub motors: Part,
    pub other: Part,
    pub areas: Vec<Area>,
}

impl Groups {
    /// Agrega un circuito con su potencia aparente. Motores y compresores
    /// resueltos van a su grupo aunque su tipo diga otra cosa.
    pub fn add(&mut self, kind: LoadKind, is_motor: bool, va: f64, continuous: bool) {
        if is_motor {
            // Los motores no son carga continua: su 125 % lo da 430-24.
            self.motors.add(va, false);
            return;
        }
        let part = match kind {
            LoadKind::Lighting => &mut self.lighting,
            LoadKind::Receptacles => &mut self.receptacles,
            LoadKind::Appliance => &mut self.appliances,
            LoadKind::KitchenEquipment => &mut self.kitchen,
            LoadKind::General | LoadKind::Motor | LoadKind::AirConditioning => &mut self.other,
        };
        part.add(va, continuous);
    }

    pub fn merge(&mut self, other: &Groups) {
        self.lighting.merge(&other.lighting);
        self.receptacles.merge(&other.receptacles);
        self.appliances.merge(&other.appliances);
        self.kitchen.merge(&other.kitchen);
        self.motors.merge(&other.motors);
        self.other.merge(&other.other);
        self.areas.extend(other.areas.iter().cloned());
    }
}

/// Carga del alimentador ya calculada.
#[derive(Debug, Clone)]
pub(crate) struct FeederDemand {
    pub lines: Vec<DemandLine>,
    pub demanded_va: f64,
    /// Parte continua de la carga calculada.
    pub continuous_va: f64,
    pub check: Check,
    pub steps: Vec<Step>,
}

fn refs(profile: &Profile, rules: &[Rule]) -> Vec<RuleReference> {
    rules.iter().map(|&r| profile.reference(r)).collect()
}

/// Potencia en VA para mostrar: sin decimales. El cálculo no se redondea.
fn va(value: f64) -> String {
    plain(value.round())
}

fn percent(value: f64) -> String {
    format!("{} %", plain(value))
}

/// Aplica una tabla de factores por escalones. Sin escalones, 100 %.
fn tiered(load: f64, tiers: &[&DemandTier]) -> (f64, String) {
    if tiers.is_empty() {
        return (load, format!("{} × 100 %", self::va(load)));
    }
    let mut demanded = 0.0;
    let mut parts = Vec::new();
    let mut from = 0.0;
    for tier in tiers {
        if load <= from {
            break;
        }
        let to = tier.up_to_va.unwrap_or(f64::INFINITY).min(load);
        let slice = to - from;
        demanded += slice * tier.percent / 100.0;
        parts.push(format!("{} × {}", va(slice), percent(tier.percent)));
        from = to;
    }
    if parts.is_empty() {
        parts.push("0".into());
    }
    (demanded, parts.join(" + "))
}

/// Datos que faltan o que impiden declarar conforme la carga del alimentador.
struct Issues(Vec<String>);

/// Carga mínima de alumbrado de las áreas (220-12) y la de contactos de
/// bancos y oficinas (220-14(k)).
struct Minimums {
    lighting_va: f64,
    lighting_detail: Vec<String>,
    receptacles_va: f64,
    receptacles_detail: Option<String>,
}

fn minimums(
    profile: &Profile,
    building: Option<&Occupancy>,
    areas: &[Area],
    issues: &mut Issues,
) -> Minimums {
    let mut m = Minimums {
        lighting_va: 0.0,
        lighting_detail: Vec::new(),
        receptacles_va: 0.0,
        receptacles_detail: None,
    };
    let mut office_m2 = 0.0;
    let mut office_unit = 0.0;
    for area in areas {
        // En vivienda toda el área va con la carga de vivienda: la Tabla
        // 220-12 excluye a las viviendas de las áreas de menor carga.
        let kind = match (building, &area.occupancy) {
            (Some(b), _) if b.dwelling => Some(b),
            (_, Some(id)) => profile.occupancy(id),
            (b, None) => b,
        };
        let Some(kind) = kind else {
            issues.0.push(match &area.occupancy {
                Some(id) => format!("el tipo de área «{id}» no está en la Tabla 220-12"),
                None => "falta el tipo de inmueble del proyecto".into(),
            });
            continue;
        };
        m.lighting_va += kind.va_per_m2 * area.m2;
        m.lighting_detail.push(format!(
            "{} m² × {} VA/m² ({})",
            plain(area.m2),
            plain(kind.va_per_m2),
            kind.label.to_lowercase()
        ));
        if let Some(unit) = profile.office_receptacle_va_per_m2(&kind.id) {
            office_m2 += area.m2;
            office_unit = unit;
        }
    }
    if office_m2 > 0.0 {
        m.receptacles_va = office_unit * office_m2;
        m.receptacles_detail = Some(format!(
            "{} m² × {} VA/m²",
            plain(office_m2),
            plain(office_unit)
        ));
    }
    m
}

/// Calcula la carga del alimentador de un tablero.
pub(crate) fn feeder_demand(
    profile: &Profile,
    occupancy: Option<&str>,
    demand: Demand,
    groups: &Groups,
) -> FeederDemand {
    let mut issues = Issues(Vec::new());
    let building = match occupancy {
        Some(id) => {
            let found = profile.occupancy(id);
            if found.is_none() {
                issues.0.push(format!(
                    "el tipo de inmueble «{id}» no está en la Tabla 220-12"
                ));
            }
            found
        }
        None => None,
    };
    let dwelling = building.is_some_and(|b| b.dwelling);
    let general_va = groups.lighting.va + groups.receptacles.va;
    if building.is_none() && occupancy.is_none() && general_va > 0.0 {
        issues
            .0
            .push("falta el tipo de inmueble del proyecto".into());
    }
    let needs_area = groups.lighting.va > 0.0 || (dwelling && general_va > 0.0);
    if needs_area && groups.areas.is_empty() && building.is_some() {
        issues
            .0
            .push("falta el área que atiende el tablero, para la carga mínima de alumbrado".into());
    }
    let min = minimums(profile, building, &groups.areas, &mut issues);
    let apply_tables = demand == Demand::Article220 && building.is_some();
    let scale = match demand {
        Demand::Custom(f) => f,
        Demand::Article220 | Demand::Full => 1.0,
    };

    let mut lines: Vec<DemandLine> = Vec::new();
    let mut continuous_va = 0.0;
    let mut notes: Vec<(String, String)> = Vec::new();
    let occupancy_label = building.map(|b| b.label.to_lowercase()).unwrap_or_default();

    // Alumbrado general, con los contactos si es vivienda (220-14(j)).
    let (general, mut general_rules, label) = if dwelling {
        let mut part = groups.lighting.clone();
        part.merge(&groups.receptacles);
        (
            part,
            vec![Rule::LightingLoad, Rule::DwellingReceptacles],
            "Alumbrado general y contactos",
        )
    } else {
        (
            groups.lighting.clone(),
            vec![Rule::LightingLoad],
            "Alumbrado general",
        )
    };
    if general.va > 0.0 || min.lighting_va > 0.0 {
        let basis = general.va.max(min.lighting_va);
        let mut minimum_note = None;
        if min.lighting_va > 0.0 {
            let detail = format!(
                "Mínima por área (220-12): {} = {} VA",
                min.lighting_detail.join(" + "),
                va(min.lighting_va)
            );
            minimum_note = Some(if min.lighting_va > general.va {
                format!(
                    "{detail}. Es mayor que la capturada ({} VA): se usa la mínima.",
                    va(general.va)
                )
            } else {
                format!(
                    "{detail}. Se usa la capturada ({} VA), que es mayor.",
                    va(general.va)
                )
            });
        }
        let (demanded, detail) = if apply_tables {
            general_rules.push(Rule::LightingDemand);
            let tiers = profile.lighting_demand_tiers(&building.expect("revisado").id);
            let (d, text) = tiered(basis, &tiers);
            (
                d,
                if tiers.is_empty() {
                    format!("{text} (Tabla 220-42, todos los demás)")
                } else {
                    format!("{text} (Tabla 220-42, {occupancy_label})")
                },
            )
        } else {
            (
                basis * scale,
                format!("{} × {}", va(basis), percent(scale * 100.0)),
            )
        };
        // Lo que se agrega para llegar a la mínima se toma como continuo,
        // salvo en vivienda: el alumbrado de un comercio u oficina opera más
        // de tres horas.
        let extra = basis - general.va;
        let continuous_basis = general.continuous_va + if dwelling { 0.0 } else { extra };
        if basis > 0.0 {
            continuous_va += demanded * continuous_basis / basis;
        }
        if let Some(note) = minimum_note {
            notes.push((label.to_string(), note));
        }
        lines.push(DemandLine {
            label: label.into(),
            connected_va: general.va,
            basis_va: basis,
            demanded_va: demanded,
            detail,
            references: refs(profile, &general_rules),
        });
    }

    // Contactos fuera de vivienda (220-14(k), 220-44).
    if !dwelling && (groups.receptacles.va > 0.0 || min.receptacles_va > 0.0) {
        let part = &groups.receptacles;
        let basis = part.va.max(min.receptacles_va);
        let mut rules = vec![Rule::ReceptacleOutletLoad];
        if let Some(text) = &min.receptacles_detail {
            rules.push(Rule::OfficeReceptacleLoad);
            notes.push((
                "Contactos".into(),
                format!(
                    "Mínima en bancos y oficinas (220-14(k)): {text} = {} VA; se usa {}.",
                    va(min.receptacles_va),
                    if min.receptacles_va > part.va {
                        "la mínima"
                    } else {
                        "la capturada, que es mayor"
                    }
                ),
            ));
        }
        let (demanded, detail) = if apply_tables {
            rules.push(Rule::ReceptacleDemand);
            let tiers: Vec<&DemandTier> = profile.receptacle_demand.iter().collect();
            let (d, text) = tiered(basis, &tiers);
            (d, format!("{text} (Tabla 220-44)"))
        } else {
            (
                basis * scale,
                format!("{} × {}", va(basis), percent(scale * 100.0)),
            )
        };
        continuous_va += demanded * part.continuous_fraction();
        lines.push(DemandLine {
            label: "Contactos".into(),
            connected_va: part.va,
            basis_va: basis,
            demanded_va: demanded,
            detail,
            references: refs(profile, &rules),
        });
    }

    // Aparatos fijos (220-53, solo en vivienda).
    if groups.appliances.va > 0.0 {
        let part = &groups.appliances;
        let count = part.items.len() as u32;
        let p = &profile.parameters;
        let (demanded, detail, rules) = if apply_tables && dwelling {
            if count >= p.appliance_demand_min_count {
                (
                    part.va * p.appliance_demand_factor,
                    format!(
                        "{} × {} ({count} aparatos fijos, 220-53)",
                        va(part.va),
                        percent(p.appliance_demand_factor * 100.0)
                    ),
                    vec![Rule::ApplianceDemand],
                )
            } else {
                (
                    part.va,
                    format!(
                        "{} × 100 % (menos de {} aparatos fijos, 220-53)",
                        va(part.va),
                        p.appliance_demand_min_count
                    ),
                    vec![Rule::ApplianceDemand],
                )
            }
        } else {
            (
                part.va * scale,
                format!("{} × {}", va(part.va), percent(scale * 100.0)),
                vec![Rule::FeederLoad],
            )
        };
        continuous_va += demanded * part.continuous_fraction();
        lines.push(DemandLine {
            label: "Aparatos fijos".into(),
            connected_va: part.va,
            basis_va: part.va,
            demanded_va: demanded,
            detail,
            references: refs(profile, &rules),
        });
    }

    // Equipo de cocina comercial (220-56).
    if groups.kitchen.va > 0.0 {
        let part = &groups.kitchen;
        let count = part.items.len() as u32;
        let (demanded, detail, rules) = match profile.kitchen_demand_percent(count) {
            Some(pct) if apply_tables && !dwelling => {
                let mut largest = part.items.clone();
                largest.sort_by(|a, b| b.total_cmp(a));
                let two: f64 = largest.iter().take(2).sum();
                let by_table = part.va * pct / 100.0;
                if two > by_table {
                    (
                        two,
                        format!(
                            "{} × {} = {} VA, menor que los dos equipos mayores: {} VA (220-56)",
                            va(part.va),
                            percent(pct),
                            va(by_table),
                            va(two)
                        ),
                        vec![Rule::KitchenDemand],
                    )
                } else {
                    (
                        by_table,
                        format!(
                            "{} × {} ({count} equipos, Tabla 220-56)",
                            va(part.va),
                            percent(pct)
                        ),
                        vec![Rule::KitchenDemand],
                    )
                }
            }
            _ => (
                part.va * scale,
                format!("{} × {}", va(part.va), percent(scale * 100.0)),
                vec![Rule::FeederLoad],
            ),
        };
        continuous_va += demanded * part.continuous_fraction();
        lines.push(DemandLine {
            label: "Equipo de cocina".into(),
            connected_va: part.va,
            basis_va: part.va,
            demanded_va: demanded,
            detail,
            references: refs(profile, &rules),
        });
    }

    for (label, part, rules) in [
        (
            "Motores y aire acondicionado",
            &groups.motors,
            vec![Rule::MotorFeederConductor, Rule::CompressorConductor],
        ),
        ("Otras cargas", &groups.other, vec![Rule::FeederLoad]),
    ] {
        if part.va <= 0.0 {
            continue;
        }
        let demanded = part.va * scale;
        continuous_va += demanded * part.continuous_fraction();
        lines.push(DemandLine {
            label: label.into(),
            connected_va: part.va,
            basis_va: part.va,
            demanded_va: demanded,
            detail: format!("{} × {}", va(part.va), percent(scale * 100.0)),
            references: refs(profile, &rules),
        });
    }

    let demanded_va: f64 = lines.iter().map(|l| l.demanded_va).sum();
    let connected_va: f64 = lines.iter().map(|l| l.connected_va).sum();

    let method = match demand {
        Demand::Article220 if building.is_some() => {
            format!("artículo 220, {occupancy_label}")
        }
        Demand::Article220 => "sin factores de demanda".into(),
        Demand::Full => "100 % de la carga, sin factores de demanda".into(),
        Demand::Custom(f) => format!("factor propio de {}", plain(f)),
    };
    if let Demand::Custom(f) = demand {
        if f < 1.0 {
            issues.0.push(format!(
                "el factor de demanda propio de {} no sale del artículo 220; justifícalo y revísalo a mano (por ejemplo con mediciones según 220-87 o 430-26)",
                plain(f)
            ));
        }
    }
    let status = if issues.0.is_empty() {
        Status::Complies
    } else {
        Status::NotEvaluated
    };
    let mut detail = format!(
        "{} VA calculados de {} VA conectados ({method})",
        va(demanded_va),
        va(connected_va)
    );
    for issue in &issues.0 {
        detail.push_str("; ");
        detail.push_str(issue);
    }
    let mut check_rules = vec![Rule::FeederLoad];
    if min.lighting_va > 0.0 || needs_area {
        check_rules.push(Rule::LightingLoad);
    }
    let check = Check {
        name: "Carga del alimentador".into(),
        status,
        detail,
        references: refs(profile, &check_rules),
    };

    let mut steps: Vec<Step> = lines
        .iter()
        .map(|l| Step {
            title: format!("Carga: {}", l.label.to_lowercase()),
            formula: None,
            substitution: format!("{} = {} VA", l.detail, va(l.demanded_va)),
            note: notes
                .iter()
                .filter(|(label, _)| *label == l.label)
                .map(|(_, n)| n.clone())
                .reduce(|a, b| format!("{a} {b}")),
            references: l.references.clone(),
        })
        .collect();
    steps.push(Step {
        title: "Carga del alimentador".into(),
        formula: Some("S = Σ carga de cada grupo con su factor de demanda".into()),
        substitution: format!(
            "S = {} = {} VA",
            lines
                .iter()
                .map(|l| va(l.demanded_va))
                .collect::<Vec<_>>()
                .join(" + "),
            va(demanded_va)
        ),
        note: Some(format!(
            "Método: {method}.{}",
            if groups.motors.va > 0.0 {
                " El 25 % adicional del motor mayor se suma en la corriente de diseño."
            } else {
                ""
            }
        )),
        references: refs(profile, &[Rule::FeederLoad]),
    });

    FeederDemand {
        lines,
        demanded_va,
        continuous_va,
        check,
        steps,
    }
}

/// Carga mínima de un circuito de contactos (220-14(i)). `None` si no aplica:
/// el circuito no es de contactos, no tiene el número de salidas o es de
/// vivienda, donde los contactos van con el alumbrado (220-14(j)).
pub(crate) fn outlet_check(
    profile: &Profile,
    occupancy: Option<&str>,
    circuit: &Circuit,
) -> Option<Check> {
    if circuit.load.kind != LoadKind::Receptacles {
        return None;
    }
    let outlets = circuit.load.outlets?;
    let building = occupancy.and_then(|id| profile.occupancy(id));
    if building.is_some_and(|b| b.dwelling) {
        return None;
    }
    let unit = profile.parameters.receptacle_outlet_va;
    let minimum = unit * f64::from(outlets);
    let pf = circuit.load.power_factor;
    let load_va = if pf > 0.0 {
        circuit.load.power_w / pf
    } else {
        0.0
    };
    let references = refs(profile, &[Rule::ReceptacleOutletLoad]);
    if building.is_none() {
        return Some(Check {
            name: "Carga de contactos".into(),
            status: Status::NotEvaluated,
            detail: "falta el tipo de inmueble del proyecto: en vivienda los contactos no llevan carga mínima por salida".into(),
            references,
        });
    }
    let passes = load_va + 1e-6 >= minimum;
    Some(Check {
        name: "Carga de contactos".into(),
        status: if passes {
            Status::Complies
        } else {
            Status::Fails
        },
        detail: format!(
            "{} VA capturados; mínimo {outlets} × {} VA = {} VA",
            va(load_va),
            plain(unit),
            va(minimum)
        ),
        references,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> &'static Profile {
        Profile::nom_001_sede_2012()
    }

    fn area(m2: f64) -> Area {
        Area {
            description: String::new(),
            occupancy: None,
            m2,
        }
    }

    /// Prueba matemática: los escalones se aplican a cada parte de la carga.
    #[test]
    fn tiers_apply_to_each_slice() {
        let p = profile();
        let dwelling = p.lighting_demand_tiers("dwellings");
        let (d, text) = tiered(10_000.0, &dwelling);
        assert!((d - (3000.0 + 7000.0 * 0.35)).abs() < 1e-9);
        assert_eq!(text, "3,000 × 100 % + 7,000 × 35 %");
        let (_, text) = tiered(5_666.667, &dwelling);
        assert_eq!(text, "3,000 × 100 % + 2,667 × 35 %");
        let (d, _) = tiered(150_000.0, &dwelling);
        assert!((d - (3000.0 + 117_000.0 * 0.35 + 30_000.0 * 0.25)).abs() < 1e-6);
        let (d, _) = tiered(2_000.0, &dwelling);
        assert_eq!(d, 2_000.0);
        let (d, _) = tiered(5_000.0, &[]);
        assert_eq!(d, 5_000.0);
    }

    /// Prueba normativa: casa de 120 m² con 5,100 VA de alumbrado y contactos.
    /// La mínima es 33 × 120 = 3,960 VA; se usa la capturada.
    #[test]
    fn dwelling_lighting_and_receptacles_share_table_220_42() {
        let mut g = Groups::default();
        g.add(LoadKind::Lighting, false, 1800.0, true);
        g.add(LoadKind::Receptacles, false, 3300.0, false);
        g.areas.push(area(120.0));
        let d = feeder_demand(profile(), Some("dwellings"), Demand::Article220, &g);
        assert_eq!(d.check.status, Status::Complies, "{}", d.check.detail);
        assert_eq!(d.lines.len(), 1);
        let expected = 3000.0 + 2100.0 * 0.35;
        assert!((d.demanded_va - expected).abs() < 1e-9);
        assert_eq!(d.lines[0].basis_va, 5100.0);
    }

    /// Prueba normativa: oficina de 200 m² con poco alumbrado capturado. La
    /// mínima de 39 VA/m² manda; los contactos llevan 11 VA/m² y la Tabla
    /// 220-44 (10 kVA al 100 %, el resto al 50 %).
    #[test]
    fn office_minimums_and_receptacle_demand() {
        let mut g = Groups::default();
        g.add(LoadKind::Lighting, false, 4000.0, true);
        g.add(LoadKind::Receptacles, false, 14_000.0, false);
        g.areas.push(area(200.0));
        let d = feeder_demand(profile(), Some("offices"), Demand::Article220, &g);
        let lighting = &d.lines[0];
        assert_eq!(lighting.basis_va, 7800.0);
        assert_eq!(lighting.demanded_va, 7800.0);
        let receptacles = &d.lines[1];
        assert_eq!(receptacles.basis_va, 14_000.0); // 2,200 VA mínima < capturada
        assert_eq!(receptacles.demanded_va, 10_000.0 + 4_000.0 * 0.5);
        // Todo el alumbrado es continuo: la parte agregada también.
        assert!((d.continuous_va - 7800.0).abs() < 1e-9);
    }

    /// Prueba normativa: cuatro aparatos fijos en vivienda van al 75 %; tres no.
    #[test]
    fn four_dwelling_appliances_take_75_percent() {
        let mut g = Groups::default();
        g.areas.push(area(80.0));
        for _ in 0..3 {
            g.add(LoadKind::Appliance, false, 1000.0, false);
        }
        let d = feeder_demand(profile(), Some("dwellings"), Demand::Article220, &g);
        assert_eq!(d.lines.last().unwrap().demanded_va, 3000.0);
        g.add(LoadKind::Appliance, false, 1000.0, false);
        let d = feeder_demand(profile(), Some("dwellings"), Demand::Article220, &g);
        assert_eq!(d.lines.last().unwrap().demanded_va, 3000.0);
        // Las mismas cuatro en una tienda: 100 %.
        let d = feeder_demand(profile(), Some("stores"), Demand::Article220, &g);
        assert_eq!(d.lines.last().unwrap().demanded_va, 4000.0);
    }

    /// Prueba normativa: la Tabla 220-56 no baja de los dos equipos mayores.
    #[test]
    fn kitchen_never_below_two_largest() {
        let mut g = Groups::default();
        for va in [10_000.0, 8_000.0, 1_000.0, 1_000.0, 1_000.0, 1_000.0] {
            g.add(LoadKind::KitchenEquipment, false, va, false);
        }
        let d = feeder_demand(profile(), Some("restaurants"), Demand::Article220, &g);
        // 22,000 × 65 % = 14,300 < 18,000
        assert_eq!(d.lines[0].demanded_va, 18_000.0);
        let mut g = Groups::default();
        for _ in 0..6 {
            g.add(LoadKind::KitchenEquipment, false, 5_000.0, false);
        }
        let d = feeder_demand(profile(), Some("restaurants"), Demand::Article220, &g);
        assert_eq!(d.lines[0].demanded_va, 30_000.0 * 0.65);
    }

    /// Sin tipo de inmueble o sin área no se declara conforme.
    #[test]
    fn missing_data_is_not_evaluated() {
        let mut g = Groups::default();
        g.add(LoadKind::Lighting, false, 1000.0, false);
        let d = feeder_demand(profile(), None, Demand::Article220, &g);
        assert_eq!(d.check.status, Status::NotEvaluated);
        assert_eq!(d.demanded_va, 1000.0);
        let d = feeder_demand(profile(), Some("stores"), Demand::Article220, &g);
        assert_eq!(d.check.status, Status::NotEvaluated);
        assert!(d.check.detail.contains("área"));
        // Solo motores: no hace falta el área.
        let mut g = Groups::default();
        g.add(LoadKind::Motor, true, 5000.0, false);
        let d = feeder_demand(profile(), Some("stores"), Demand::Article220, &g);
        assert_eq!(d.check.status, Status::Complies);
    }

    #[test]
    fn custom_factors_below_one_are_not_evaluated() {
        let mut g = Groups::default();
        g.add(LoadKind::General, false, 10_000.0, false);
        let d = feeder_demand(profile(), Some("stores"), Demand::Custom(0.8), &g);
        assert_eq!(d.demanded_va, 8000.0);
        assert_eq!(d.check.status, Status::NotEvaluated);
        let d = feeder_demand(profile(), Some("stores"), Demand::Full, &g);
        assert_eq!(d.demanded_va, 10_000.0);
        assert_eq!(d.check.status, Status::Complies);
    }

    /// Las áreas de pasillos y bodegas llevan su propia carga, salvo en vivienda.
    #[test]
    fn area_types_inside_a_building() {
        let mut g = Groups::default();
        g.add(LoadKind::Lighting, false, 100.0, true);
        g.areas.push(area(100.0));
        g.areas.push(Area {
            description: "Pasillos".into(),
            occupancy: Some("halls_corridors".into()),
            m2: 50.0,
        });
        let d = feeder_demand(profile(), Some("schools"), Demand::Article220, &g);
        assert_eq!(d.lines[0].basis_va, 33.0 * 100.0 + 6.0 * 50.0);
        let d = feeder_demand(profile(), Some("dwellings"), Demand::Article220, &g);
        assert_eq!(d.lines[0].basis_va, 33.0 * 150.0);
    }
}
