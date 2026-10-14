//! Dimensionamiento de tubo conduit (capítulo 10, Tablas 1, 4, 5 y 8).

use ieestudio_core::units::percent;
use ieestudio_core::{ConduitType, Error, InsulationGroup, Result, Size};
use ieestudio_rules::tables::ConduitRow;
use ieestudio_rules::{Profile, Rule};

/// Cubierta de un conductor dentro del tubo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Covering {
    /// Conductor aislado: dimensiones de la Tabla 5.
    Insulated(InsulationGroup),
    /// Conductor desnudo: dimensiones de la Tabla 8.
    Bare,
}

/// Conjunto de conductores iguales dentro del tubo.
#[derive(Debug, Clone, PartialEq)]
pub struct ConductorGroup {
    pub count: u32,
    pub size: Size,
    pub covering: Covering,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConduitInput {
    pub groups: Vec<ConductorGroup>,
    pub conduit: ConduitType,
    /// Tramo de 60 cm o menos entre cajas o gabinetes.
    pub nipple: bool,
}

/// Área y diámetro de cada grupo, para mostrar el procedimiento.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupArea {
    pub count: u32,
    pub size: Size,
    pub diameter_mm: f64,
    pub unit_area_mm2: f64,
    pub total_area_mm2: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConduitResult {
    pub groups: Vec<GroupArea>,
    pub conductor_count: u32,
    pub conductor_area_mm2: f64,
    /// Fracción de ocupación permitida: 0.53, 0.31, 0.40 o 0.60.
    pub allowed_fill_fraction: f64,
    /// Tubo seleccionado.
    pub conduit: ConduitRow,
    /// Área permitida en el tubo seleccionado, en mm².
    pub allowed_area_mm2: f64,
    /// Ocupación real del tubo seleccionado, en porcentaje de su área total.
    pub fill_percent: f64,
    /// Relación de atascamiento, cuando aplica (tres conductores iguales).
    pub jam_ratio: Option<f64>,
    /// El tubo se subió un tamaño por riesgo de atascamiento.
    pub upsized_for_jamming: bool,
}

/// Área permitida en un tubo para una fracción de ocupación.
///
/// Se usan las áreas impresas en la Tabla 4; solo si la fracción no es una de
/// sus columnas se calcula sobre el área total.
fn allowed_area(row: &ConduitRow, fraction: f64) -> f64 {
    let columns = [
        (0.60, row.area_60_mm2),
        (0.53, row.area_53_mm2),
        (0.31, row.area_31_mm2),
        (0.40, row.area_40_mm2),
    ];
    columns
        .iter()
        .find(|(f, _)| (f - fraction).abs() < 1e-9)
        .map_or(fraction * row.area_100_mm2, |&(_, area)| area)
}

/// Selecciona el menor tubo que admite los conductores.
pub fn size_conduit(profile: &Profile, input: &ConduitInput) -> Result<ConduitResult> {
    let mut groups = Vec::new();
    for g in input.groups.iter().filter(|g| g.count > 0) {
        let (diameter_mm, unit_area_mm2) = match g.covering {
            Covering::Insulated(ins) => {
                let c = profile.insulated(ins, &g.size)?;
                (c.diameter_mm, c.area_mm2)
            }
            Covering::Bare => {
                let c = profile.conductor(&g.size)?;
                (c.bare_diameter_mm, c.bare_area_mm2)
            }
        };
        groups.push(GroupArea {
            count: g.count,
            size: g.size.clone(),
            diameter_mm,
            unit_area_mm2,
            total_area_mm2: f64::from(g.count) * unit_area_mm2,
        });
    }
    let conductor_count: u32 = groups.iter().map(|g| g.count).sum();
    if conductor_count == 0 {
        return Err(Error::InvalidInput("el tubo no tiene conductores".into()));
    }
    let conductor_area_mm2: f64 = groups.iter().map(|g| g.total_area_mm2).sum();

    let allowed_fill_fraction = if input.nipple {
        profile.parameters.nipple_fill_fraction
    } else {
        profile
            .fill
            .iter()
            .find(|r| {
                conductor_count >= r.conductors_min
                    && r.conductors_max.map_or(true, |max| conductor_count <= max)
            })
            .map(|r| r.fill_fraction)
            .ok_or(Error::NotInTable {
                table: profile.clause(Rule::ConduitFill),
                what: format!("{conductor_count} conductores"),
            })?
    };

    // El atascamiento se evalúa con tres conductores del mismo diámetro.
    let jam_diameter = (conductor_count == 3
        && groups
            .iter()
            .all(|g| g.diameter_mm == groups[0].diameter_mm))
    .then(|| groups[0].diameter_mm);
    let jam_ratio_of = |row: &ConduitRow| jam_diameter.map(|d| row.internal_diameter_mm / d);
    let jams = |row: &ConduitRow| {
        jam_ratio_of(row).is_some_and(|r| {
            (profile.parameters.jam_ratio_min..=profile.parameters.jam_ratio_max).contains(&r)
        })
    };

    let fits: Vec<&ConduitRow> = profile
        .conduit_rows(input.conduit)
        .filter(|row| conductor_area_mm2 <= allowed_area(row, allowed_fill_fraction))
        .collect();
    let no_fit = || Error::NoConduitFits {
        conduit: input.conduit.to_string(),
        area_mm2: conductor_area_mm2,
    };
    let first = *fits.first().ok_or_else(no_fit)?;
    let chosen = *fits.iter().find(|row| !jams(row)).ok_or_else(no_fit)?;

    Ok(ConduitResult {
        groups,
        conductor_count,
        conductor_area_mm2,
        allowed_fill_fraction,
        allowed_area_mm2: allowed_area(chosen, allowed_fill_fraction),
        fill_percent: percent(conductor_area_mm2, chosen.area_100_mm2),
        jam_ratio: jam_ratio_of(chosen),
        upsized_for_jamming: chosen != first,
        conduit: chosen.clone(),
    })
}
