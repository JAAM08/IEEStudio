//! Intercambio de datos de VoltForge con otros programas.
//!
//! El cuadro de cargas se exporta a un libro de Excel con tres hojas:
//! «Tableros» y «Cuadro de cargas», con los datos que captura el usuario, y
//! «Resultados», con lo que calculó el programa. Las dos primeras se pueden
//! llenar o corregir en Excel y volver a importar: así se captura un proyecto
//! grande en una hoja de cálculo.
//!
//! Las columnas se reconocen por su encabezado, no por su posición, y una
//! columna que falta toma el valor del circuito existente o el de omisión.

use std::collections::HashMap;
use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Range, Reader, Xlsx};
use rust_xlsxwriter::{DataValidation, Format, FormatAlign, Workbook, Worksheet, XlsxError};
use voltforge_core::{CircuitKind, Size};
use voltforge_project::{
    Circuit, CompressorLoad, Demand, Feeder, Load, LoadKind, MotorLoad, Overrides, Panel, Phase,
    Project, ProjectResult, RunResult,
};

pub const PANELS_SHEET: &str = "Tableros";
pub const CIRCUITS_SHEET: &str = "Cuadro de cargas";
pub const RESULTS_SHEET: &str = "Resultados";

const LOAD_KINDS: [LoadKind; 7] = [
    LoadKind::General,
    LoadKind::Lighting,
    LoadKind::Receptacles,
    LoadKind::Appliance,
    LoadKind::KitchenEquipment,
    LoadKind::Motor,
    LoadKind::AirConditioning,
];

fn kind_label(kind: CircuitKind) -> &'static str {
    match kind {
        CircuitKind::LineToNeutral => "1 fase",
        CircuitKind::LineToLine => "2 fases",
        CircuitKind::ThreePhase => "3 fases",
    }
}

fn parse_kind(text: &str) -> Option<CircuitKind> {
    let t = text.trim().to_lowercase().replace(' ', "");
    match t.as_str() {
        "1fase" | "1f" | "1" | "1f-2h" | "monofásico" | "monofasico" => {
            Some(CircuitKind::LineToNeutral)
        }
        "2fases" | "2f" | "2" | "2f-3h" | "2f-2h" | "bifásico" | "bifasico" => {
            Some(CircuitKind::LineToLine)
        }
        "3fases" | "3f" | "3" | "3f-3h" | "3f-4h" | "trifásico" | "trifasico" => {
            Some(CircuitKind::ThreePhase)
        }
        _ => None,
    }
}

fn parse_load_kind(text: &str) -> Option<LoadKind> {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return Some(LoadKind::General);
    }
    LOAD_KINDS
        .into_iter()
        .find(|k| k.label().to_lowercase() == t)
}

fn phases_label(phases: &[Phase]) -> String {
    phases.iter().map(|p| format!("{p:?}")).collect()
}

fn parse_phases(text: &str) -> Option<Vec<Phase>> {
    let mut out = Vec::new();
    for c in text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != '-')
    {
        let phase = match c.to_ascii_uppercase() {
            'A' | 'R' => Phase::A,
            'B' | 'S' => Phase::B,
            'C' | 'T' => Phase::C,
            _ => return None,
        };
        if !out.contains(&phase) {
            out.push(phase);
        }
    }
    Some(out)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "Sí"
    } else {
        "No"
    }
}

fn parse_bool(text: &str) -> Option<bool> {
    match text.trim().to_lowercase().as_str() {
        "sí" | "si" | "s" | "x" | "1" | "verdadero" | "true" | "yes" => Some(true),
        "no" | "n" | "0" | "" | "falso" | "false" => Some(false),
        _ => None,
    }
}

// ------------------------------------------------------------------ exportar

/// Columnas de la hoja de tableros.
const PANEL_COLUMNS: [(&str, f64); 7] = [
    ("Tablero", 10.0),
    ("Nombre", 28.0),
    ("Se alimenta de", 16.0),
    ("Conexión", 10.0),
    ("Longitud del alimentador m", 14.0),
    ("Área m²", 10.0),
    ("Cap. interruptiva kA", 12.0),
];

/// Columnas de la hoja del cuadro de cargas.
const CIRCUIT_COLUMNS: [(&str, f64); 18] = [
    ("Tablero", 10.0),
    ("ID", 8.0),
    ("Descripción", 30.0),
    ("Tipo de carga", 18.0),
    ("Conexión", 10.0),
    ("Fases", 7.0),
    ("Potencia W", 11.0),
    ("Factor de potencia", 10.0),
    ("Carga continua", 9.0),
    ("Salidas de contacto", 10.0),
    ("Longitud m", 10.0),
    ("Motor hp", 9.0),
    ("Corriente de placa A", 11.0),
    ("RLA A", 8.0),
    ("MCA A", 8.0),
    ("MOP A", 8.0),
    ("Calibre fijado", 10.0),
    ("Interruptor fijado A", 11.0),
];

const RESULT_COLUMNS: [(&str, f64); 15] = [
    ("Tablero", 10.0),
    ("ID", 8.0),
    ("Descripción", 30.0),
    ("Carga W", 10.0),
    ("I A", 9.0),
    ("Id A", 9.0),
    ("Interruptor", 11.0),
    ("Calibre", 9.0),
    ("Tierra", 8.0),
    ("Tubo", 12.0),
    ("e %", 8.0),
    ("Σe %", 8.0),
    ("Icc kA", 8.0),
    ("Estado", 12.0),
    ("Observaciones", 60.0),
];

fn header(sheet: &mut Worksheet, columns: &[(&str, f64)]) -> Result<(), XlsxError> {
    let bold = Format::new()
        .set_bold()
        .set_background_color("#E9ECEF")
        .set_align(FormatAlign::Center)
        .set_text_wrap();
    for (c, (name, width)) in columns.iter().enumerate() {
        sheet.write_string_with_format(0, c as u16, *name, &bold)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    sheet.set_row_height(0, 30)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn number(sheet: &mut Worksheet, row: u32, col: u16, value: Option<f64>) -> Result<(), XlsxError> {
    if let Some(v) = value {
        sheet.write_number(row, col, v)?;
    }
    Ok(())
}

fn text(sheet: &mut Worksheet, row: u32, col: u16, value: &str) -> Result<(), XlsxError> {
    if !value.is_empty() {
        sheet.write_string(row, col, value)?;
    }
    Ok(())
}

/// Escribe el libro de Excel del proyecto. Con `result`, agrega la hoja de
/// resultados.
pub fn export_xlsx(project: &Project, result: Option<&ProjectResult>) -> Result<Vec<u8>, String> {
    build(project, result).map_err(|e| format!("no se pudo escribir el libro de Excel: {e}"))
}

fn build(project: &Project, result: Option<&ProjectResult>) -> Result<Vec<u8>, XlsxError> {
    let mut book = Workbook::new();

    let sheet = book.add_worksheet();
    sheet.set_name(PANELS_SHEET)?;
    header(sheet, &PANEL_COLUMNS)?;
    for (i, panel) in project.panels.iter().enumerate() {
        let r = i as u32 + 1;
        text(sheet, r, 0, &panel.id)?;
        text(sheet, r, 1, &panel.name)?;
        text(sheet, r, 2, panel.parent.as_deref().unwrap_or(""))?;
        text(sheet, r, 3, kind_label(panel.kind))?;
        number(sheet, r, 4, Some(panel.feeder.length_m))?;
        let area: f64 = panel.areas.iter().map(|a| a.m2).sum();
        number(sheet, r, 5, (area > 0.0).then_some(area))?;
        number(sheet, r, 6, panel.interrupting_ka)?;
    }
    let kinds = DataValidation::new().allow_list_strings(&["1 fase", "2 fases", "3 fases"])?;
    sheet.add_data_validation(1, 3, 2000, 3, &kinds)?;

    let sheet = book.add_worksheet();
    sheet.set_name(CIRCUITS_SHEET)?;
    header(sheet, &CIRCUIT_COLUMNS)?;
    let mut r = 1u32;
    for panel in &project.panels {
        for c in &panel.circuits {
            let load = &c.load;
            text(sheet, r, 0, &panel.id)?;
            text(sheet, r, 1, &c.id)?;
            text(sheet, r, 2, &c.name)?;
            text(sheet, r, 3, load.kind.label())?;
            text(sheet, r, 4, kind_label(c.kind))?;
            text(sheet, r, 5, &phases_label(&c.phases))?;
            number(sheet, r, 6, Some(load.power_w))?;
            number(sheet, r, 7, Some(load.power_factor))?;
            text(sheet, r, 8, yes_no(load.continuous))?;
            number(sheet, r, 9, load.outlets.map(f64::from))?;
            number(sheet, r, 10, Some(c.length_m))?;
            if let (LoadKind::Motor, Some(m)) = (load.kind, &load.motor) {
                text(sheet, r, 11, &m.hp)?;
                number(sheet, r, 12, m.nameplate_a)?;
            }
            if let (LoadKind::AirConditioning, Some(plate)) = (load.kind, &load.compressor) {
                number(sheet, r, 13, plate.rated_load_a)?;
                number(sheet, r, 14, plate.minimum_circuit_a)?;
                number(sheet, r, 15, plate.maximum_protection_a)?;
            }
            text(
                sheet,
                r,
                16,
                c.overrides.size.as_ref().map_or("", |s| s.0.as_str()),
            )?;
            number(sheet, r, 17, c.overrides.breaker_a)?;
            r += 1;
        }
    }
    let labels: Vec<&str> = LOAD_KINDS.iter().map(|k| k.label()).collect();
    let load_kinds = DataValidation::new().allow_list_strings(&labels)?;
    sheet.add_data_validation(1, 3, 5000, 3, &load_kinds)?;
    let kinds = DataValidation::new().allow_list_strings(&["1 fase", "2 fases", "3 fases"])?;
    sheet.add_data_validation(1, 4, 5000, 4, &kinds)?;
    let yes = DataValidation::new().allow_list_strings(&["Sí", "No"])?;
    sheet.add_data_validation(1, 8, 5000, 8, &yes)?;

    if let Some(result) = result {
        let sheet = book.add_worksheet();
        sheet.set_name(RESULTS_SHEET)?;
        header(sheet, &RESULT_COLUMNS)?;
        let mut r = 1u32;
        for panel in &result.panels {
            let rows = std::iter::once(("ALIM".to_string(), &panel.feeder))
                .chain(panel.circuits.iter().map(|c: &RunResult| (c.id.clone(), c)));
            for (id, item) in rows {
                text(sheet, r, 0, &panel.id)?;
                text(sheet, r, 1, &id)?;
                text(sheet, r, 2, &item.name)?;
                number(sheet, r, 3, Some(item.load_w.round()))?;
                if let Some(run) = &item.run {
                    number(sheet, r, 4, Some((run.nominal_a * 100.0).round() / 100.0))?;
                    number(sheet, r, 5, Some((run.design_a * 100.0).round() / 100.0))?;
                    text(
                        sheet,
                        r,
                        6,
                        &format!("{}P-{:.0} A", run.poles, run.breaker_a),
                    )?;
                    let size = if run.conductors_per_phase > 1 {
                        format!("{}×{}", run.conductors_per_phase, run.size.0)
                    } else {
                        run.size.0.clone()
                    };
                    text(sheet, r, 7, &size)?;
                    text(sheet, r, 8, &run.grounding_size.0)?;
                    if let Some(c) = &run.conduit {
                        text(sheet, r, 9, &c.label())?;
                    }
                    number(
                        sheet,
                        r,
                        10,
                        Some((run.drop_percent * 100.0).round() / 100.0),
                    )?;
                    number(
                        sheet,
                        r,
                        11,
                        Some((run.cumulative_drop_percent * 100.0).round() / 100.0),
                    )?;
                    number(
                        sheet,
                        r,
                        12,
                        run.fault_ka.map(|k| (k * 100.0).round() / 100.0),
                    )?;
                }
                text(sheet, r, 13, item.status.label())?;
                let notes: Vec<String> = item
                    .checks
                    .iter()
                    .filter(|c| c.status != voltforge_project::Status::Complies)
                    .map(|c| format!("{}: {}", c.name, c.detail))
                    .collect();
                text(sheet, r, 14, &notes.join("; "))?;
                r += 1;
            }
        }
    }
    book.save_to_buffer()
}

// ------------------------------------------------------------------ importar

/// Lo que hizo una importación.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportReport {
    pub panels_created: Vec<String>,
    pub panels_updated: usize,
    pub circuits_added: usize,
    pub circuits_updated: usize,
    /// Filas que no se importaron, con la razón.
    pub errors: Vec<String>,
}

impl ImportReport {
    /// Resumen en una línea para mostrar al usuario.
    pub fn summary(&self) -> String {
        let mut parts = vec![format!(
            "{} circuito{} nuevo{}, {} actualizado{}",
            self.circuits_added,
            plural(self.circuits_added),
            plural(self.circuits_added),
            self.circuits_updated,
            plural(self.circuits_updated)
        )];
        if !self.panels_created.is_empty() {
            parts.push(format!(
                "tableros nuevos: {}",
                self.panels_created.join(", ")
            ));
        }
        if !self.errors.is_empty() {
            parts.push(format!(
                "{} fila{} con errores",
                self.errors.len(),
                plural(self.errors.len())
            ));
        }
        parts.join("; ")
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Una hoja leída: encabezados normalizados y filas como texto.
struct Sheet {
    columns: HashMap<String, usize>,
    rows: Vec<Vec<String>>,
}

fn normalize(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => if *b { "Sí" } else { "No" }.to_string(),
        other => other.to_string(),
    }
}

impl Sheet {
    fn from_range(range: &Range<Data>) -> Sheet {
        let mut rows = range.rows();
        let columns = rows
            .next()
            .map(|h| {
                h.iter()
                    .enumerate()
                    .map(|(i, c)| (normalize(&cell_text(c)), i))
                    .filter(|(name, _)| !name.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let rows = rows
            .map(|r| r.iter().map(cell_text).collect::<Vec<_>>())
            .collect();
        Sheet { columns, rows }
    }

    fn has(&self, column: &str) -> bool {
        self.columns.contains_key(&normalize(column))
    }

    /// Valor de una columna en una fila. `None` si la columna no existe;
    /// `Some("")` si existe y la celda está vacía.
    fn get<'a>(&self, row: &'a [String], column: &str) -> Option<&'a str> {
        let i = *self.columns.get(&normalize(column))?;
        Some(row.get(i).map_or("", |s| s.as_str()))
    }
}

fn parse_number(text: &str, what: &str) -> Result<f64, String> {
    let t = text.trim().replace(',', ".");
    t.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("{what}: «{text}» no es un número"))
}

/// Número opcional: vacío es `None`.
fn optional_number(text: Option<&str>, what: &str) -> Result<Option<Option<f64>>, String> {
    match text {
        None => Ok(None),
        Some(t) if t.trim().is_empty() => Ok(Some(None)),
        Some(t) => parse_number(t, what).map(|v| Some(Some(v))),
    }
}

/// Lee un libro de Excel y lo aplica al proyecto: agrega o actualiza tableros
/// y circuitos por su ID. Nunca borra: lo que no aparece en el libro queda
/// como estaba.
pub fn import_xlsx(project: &mut Project, bytes: &[u8]) -> Result<ImportReport, String> {
    let mut book: Xlsx<_> = open_workbook_from_rs(Cursor::new(bytes))
        .map_err(|e| format!("no se pudo leer el libro de Excel: {e}"))?;
    let names = book.sheet_names();
    let find = |wanted: &str| {
        names
            .iter()
            .find(|n| normalize(n) == normalize(wanted))
            .cloned()
    };
    let mut report = ImportReport::default();

    if let Some(name) = find(PANELS_SHEET) {
        let range = book
            .worksheet_range(&name)
            .map_err(|e| format!("no se pudo leer la hoja «{name}»: {e}"))?;
        import_panels(project, &Sheet::from_range(&range), &mut report);
    }
    let circuits = find(CIRCUITS_SHEET)
        .or_else(|| names.first().cloned())
        .ok_or("el libro no tiene hojas")?;
    let range = book
        .worksheet_range(&circuits)
        .map_err(|e| format!("no se pudo leer la hoja «{circuits}»: {e}"))?;
    let sheet = Sheet::from_range(&range);
    for required in ["Tablero", "ID"] {
        if !sheet.has(required) {
            return Err(format!(
                "la hoja «{circuits}» no tiene la columna «{required}»; exporta un cuadro de cargas para ver el formato"
            ));
        }
    }
    let profile = voltforge_rules::Profile::by_id(&project.profile)
        .ok_or_else(|| format!("perfil normativo desconocido: «{}»", project.profile))?;
    import_circuits(project, profile, &sheet, &mut report);
    Ok(report)
}

fn new_panel(id: &str) -> Panel {
    Panel {
        id: id.to_string(),
        name: format!("Tablero {id}"),
        parent: None,
        kind: CircuitKind::ThreePhase,
        feeder: Feeder {
            length_m: 10.0,
            demand: Demand::Article220,
            demand_factor: None,
            has_neutral: true,
            interrupting_ka: None,
            overrides: Overrides::default(),
        },
        areas: vec![],
        interrupting_ka: None,
        circuits: vec![],
    }
}

fn import_panels(project: &mut Project, sheet: &Sheet, report: &mut ImportReport) {
    for (n, row) in sheet.rows.iter().enumerate() {
        let line = n + 2;
        let Some(id) = sheet.get(row, "Tablero").map(str::trim) else {
            report
                .errors
                .push("Tableros: falta la columna «Tablero»".into());
            return;
        };
        if id.is_empty() {
            continue;
        }
        let parsed = (|| -> Result<_, String> {
            let kind = match sheet.get(row, "Conexión") {
                Some(t) if !t.trim().is_empty() => {
                    Some(parse_kind(t).ok_or_else(|| format!("conexión desconocida «{t}»"))?)
                }
                _ => None,
            };
            let length = optional_number(
                sheet.get(row, "Longitud del alimentador m"),
                "longitud del alimentador",
            )?;
            let area = optional_number(sheet.get(row, "Área m²"), "área")?;
            let ka = optional_number(
                sheet.get(row, "Cap. interruptiva kA"),
                "capacidad interruptiva",
            )?;
            Ok((kind, length, area, ka))
        })();
        let (kind, length, area, ka) = match parsed {
            Ok(v) => v,
            Err(e) => {
                report.errors.push(format!("Tableros, fila {line}: {e}"));
                continue;
            }
        };
        let index = match project.panels.iter().position(|p| p.id == id) {
            Some(i) => {
                report.panels_updated += 1;
                i
            }
            None => {
                project.panels.push(new_panel(id));
                report.panels_created.push(id.to_string());
                project.panels.len() - 1
            }
        };
        let panel = &mut project.panels[index];
        if let Some(name) = sheet.get(row, "Nombre").filter(|t| !t.trim().is_empty()) {
            panel.name = name.trim().to_string();
        }
        if let Some(parent) = sheet.get(row, "Se alimenta de") {
            let parent = parent.trim();
            panel.parent = (!parent.is_empty() && parent != id).then(|| parent.to_string());
        }
        if let Some(kind) = kind {
            panel.kind = kind;
        }
        if let Some(Some(length)) = length {
            panel.feeder.length_m = length;
        }
        if let Some(area) = area {
            let total: f64 = panel.areas.iter().map(|a| a.m2).sum();
            match area {
                // Sin cambio: se conservan las áreas por tipo.
                Some(m2) if (m2 - total).abs() < 0.01 => {}
                Some(m2) if m2 > 0.0 => {
                    if panel.areas.len() > 1 {
                        report.errors.push(format!(
                            "Tableros, fila {line}: {id} tiene varias áreas de distinto tipo; el área total no se cambió, ajústala en la aplicación"
                        ));
                    } else {
                        panel.areas = vec![voltforge_project::Area {
                            description: panel
                                .areas
                                .first()
                                .map(|a| a.description.clone())
                                .unwrap_or_default(),
                            occupancy: panel.areas.first().and_then(|a| a.occupancy.clone()),
                            m2,
                        }];
                    }
                }
                _ => panel.areas.clear(),
            }
        }
        if let Some(ka) = ka {
            panel.interrupting_ka = ka;
        }
    }
}

fn import_circuits(
    project: &mut Project,
    profile: &voltforge_rules::Profile,
    sheet: &Sheet,
    report: &mut ImportReport,
) {
    for (n, row) in sheet.rows.iter().enumerate() {
        let line = n + 2;
        let panel_id = sheet.get(row, "Tablero").unwrap_or("").trim().to_string();
        let id = sheet.get(row, "ID").unwrap_or("").trim().to_string();
        if panel_id.is_empty() && id.is_empty() {
            continue;
        }
        if panel_id.is_empty() || id.is_empty() {
            report.errors.push(format!(
                "Cuadro de cargas, fila {line}: faltan el tablero o el ID del circuito"
            ));
            continue;
        }
        let panel_index = match project.panels.iter().position(|p| p.id == panel_id) {
            Some(i) => i,
            None => {
                project.panels.push(new_panel(&panel_id));
                report.panels_created.push(panel_id.clone());
                project.panels.len() - 1
            }
        };
        let existing = project.panels[panel_index]
            .circuits
            .iter()
            .position(|c| c.id == id);
        let mut circuit = match existing {
            Some(k) => project.panels[panel_index].circuits[k].clone(),
            None => Circuit {
                id: id.clone(),
                name: id.clone(),
                kind: CircuitKind::LineToNeutral,
                phases: vec![],
                load: Load {
                    power_w: 0.0,
                    power_factor: 0.9,
                    continuous: false,
                    kind: LoadKind::General,
                    outlets: None,
                    motor: None,
                    compressor: None,
                },
                length_m: 10.0,
                has_neutral: false,
                overrides: Overrides::default(),
            },
        };
        match apply_row(&mut circuit, profile, sheet, row) {
            Ok(()) => match existing {
                Some(k) => {
                    project.panels[panel_index].circuits[k] = circuit;
                    report.circuits_updated += 1;
                }
                None => {
                    project.panels[panel_index].circuits.push(circuit);
                    report.circuits_added += 1;
                }
            },
            Err(e) => report
                .errors
                .push(format!("Cuadro de cargas, fila {line} ({id}): {e}")),
        }
    }
}

/// Aplica una fila al circuito. Las columnas que no existen no cambian nada.
fn apply_row(
    c: &mut Circuit,
    profile: &voltforge_rules::Profile,
    sheet: &Sheet,
    row: &[String],
) -> Result<(), String> {
    if let Some(name) = sheet.get(row, "Descripción").filter(|t| !t.is_empty()) {
        c.name = name.to_string();
    }
    if let Some(t) = sheet.get(row, "Tipo de carga") {
        c.load.kind = parse_load_kind(t).ok_or_else(|| {
            format!(
                "tipo de carga desconocido «{t}»; usa {}",
                LOAD_KINDS
                    .iter()
                    .map(|k| k.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    }
    if let Some(t) = sheet.get(row, "Conexión").filter(|t| !t.is_empty()) {
        let kind = parse_kind(t).ok_or_else(|| format!("conexión desconocida «{t}»"))?;
        if kind != c.kind {
            c.phases.clear();
        }
        c.kind = kind;
    }
    if let Some(t) = sheet.get(row, "Fases") {
        c.phases = parse_phases(t).ok_or_else(|| format!("fases desconocidas «{t}»"))?;
    }
    if let Some(Some(w)) = optional_number(sheet.get(row, "Potencia W"), "potencia")? {
        if w < 0.0 {
            return Err("la potencia no puede ser negativa".into());
        }
        c.load.power_w = w;
    }
    if let Some(Some(pf)) =
        optional_number(sheet.get(row, "Factor de potencia"), "factor de potencia")?
    {
        if !(0.1..=1.0).contains(&pf) {
            return Err(format!("factor de potencia fuera de 0.1 a 1: {pf}"));
        }
        c.load.power_factor = pf;
    }
    if let Some(t) = sheet.get(row, "Carga continua") {
        c.load.continuous =
            parse_bool(t).ok_or_else(|| format!("carga continua: «{t}» no es Sí ni No"))?;
    }
    if let Some(outlets) = optional_number(sheet.get(row, "Salidas de contacto"), "salidas")? {
        c.load.outlets = outlets.filter(|n| *n > 0.0).map(|n| n.round() as u32);
    }
    if let Some(Some(l)) = optional_number(sheet.get(row, "Longitud m"), "longitud")? {
        if l < 0.0 {
            return Err("la longitud no puede ser negativa".into());
        }
        c.length_m = l;
    }
    match c.load.kind {
        LoadKind::Motor => {
            let hp = sheet.get(row, "Motor hp").map(str::trim).unwrap_or("");
            if hp.is_empty() && c.load.motor.is_none() {
                return Err("falta la potencia del motor en la columna «Motor hp»".into());
            }
            let plate =
                optional_number(sheet.get(row, "Corriente de placa A"), "corriente de placa")?;
            let motor = c.load.motor.get_or_insert_with(|| MotorLoad {
                hp: "1".into(),
                motor_type: Default::default(),
                nameplate_a: None,
                service_factor_marked: false,
            });
            if !hp.is_empty() {
                motor.hp = hp.to_string();
            }
            if let Some(a) = plate {
                motor.nameplate_a = a;
            }
        }
        LoadKind::AirConditioning => {
            let plate = c
                .load
                .compressor
                .get_or_insert_with(CompressorLoad::default);
            if let Some(v) = optional_number(sheet.get(row, "RLA A"), "RLA")? {
                plate.rated_load_a = v;
            }
            if let Some(v) = optional_number(sheet.get(row, "MCA A"), "MCA")? {
                plate.minimum_circuit_a = v;
            }
            if let Some(v) = optional_number(sheet.get(row, "MOP A"), "MOP")? {
                plate.maximum_protection_a = v;
            }
        }
        _ => {}
    }
    if let Some(t) = sheet.get(row, "Calibre fijado") {
        let t = t
            .trim()
            .trim_end_matches("AWG")
            .trim_end_matches("kcmil")
            .trim();
        c.overrides.size = if t.is_empty() {
            None
        } else {
            let size = Size::new(t);
            profile
                .conductor(&size)
                .map_err(|_| format!("calibre fijado desconocido «{t}»"))?;
            Some(size)
        };
    }
    if let Some(v) = optional_number(sheet.get(row, "Interruptor fijado A"), "interruptor")? {
        c.overrides.breaker_a = v;
    }
    Ok(())
}
