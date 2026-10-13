//! El cuadro de cargas va y vuelve de Excel sin perder datos.

use rust_xlsxwriter::Workbook;
use voltforge_exchange::{export_xlsx, import_xlsx, CIRCUITS_SHEET};
use voltforge_project::{calculate, example, LoadKind, Project};

fn empty_like(project: &Project) -> Project {
    let mut empty = project.clone();
    empty.panels.clear();
    empty
}

#[test]
fn export_and_import_round_trip() {
    let house = example::house();
    let result = calculate(&house).unwrap();
    let bytes = export_xlsx(&house, Some(&result)).unwrap();
    assert!(bytes.starts_with(b"PK"), "un .xlsx es un zip");

    // Al importarlo en un proyecto sin tableros, se crean el tablero y sus
    // circuitos con los mismos datos.
    let mut target = empty_like(&house);
    let report = import_xlsx(&mut target, &bytes).unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.circuits_added, house.panels[0].circuits.len());
    assert_eq!(report.panels_created, vec!["TG-1".to_string()]);
    let (a, b) = (&house.panels[0], &target.panels[0]);
    assert_eq!(a.name, b.name);
    assert_eq!(a.feeder.length_m, b.feeder.length_m);
    assert_eq!(a.interrupting_ka, b.interrupting_ka);
    for (x, y) in a.circuits.iter().zip(&b.circuits) {
        assert_eq!(x.id, y.id);
        assert_eq!(x.name, y.name);
        assert_eq!(x.kind, y.kind);
        assert_eq!(x.phases, y.phases);
        assert_eq!(x.load.kind, y.load.kind);
        assert_eq!(x.load.continuous, y.load.continuous);
        assert!((x.load.power_w - y.load.power_w).abs() < 1e-9);
        assert!((x.length_m - y.length_m).abs() < 1e-9);
        assert_eq!(x.load.motor, y.load.motor);
        assert_eq!(x.load.compressor, y.load.compressor);
        assert_eq!(x.overrides.size, y.overrides.size);
    }

    // Al importarlo sobre el mismo proyecto todo se actualiza y nada cambia.
    let mut same = house.clone();
    let report = import_xlsx(&mut same, &bytes).unwrap();
    assert_eq!(report.circuits_updated, house.panels[0].circuits.len());
    assert_eq!(report.circuits_added, 0);
    assert_eq!(same, house);
}

/// Una hoja escrita a mano: columnas en otro orden, sin las opcionales, con
/// abreviaturas y con filas malas que se informan sin detener la importación.
#[test]
fn a_hand_written_sheet_is_imported_with_its_errors() {
    let mut book = Workbook::new();
    let sheet = book.add_worksheet();
    sheet.set_name(CIRCUITS_SHEET).unwrap();
    let rows: [[&str; 7]; 6] = [
        [
            "ID",
            "Tablero",
            "Potencia W",
            "Tipo de carga",
            "Conexión",
            "Fases",
            "Motor hp",
        ],
        ["L-1", "TA", "1500", "Alumbrado", "1F", "a", ""],
        ["M-1", "TA", "", "Motor", "3F", "ABC", "7-1/2"],
        ["X-1", "TA", "mil", "General", "1 fase", "B", ""],
        ["X-2", "TA", "100", "Calefactor", "1 fase", "B", ""],
        ["M-2", "TA", "", "Motor", "3F", "ABC", ""],
    ];
    for (r, row) in rows.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            sheet.write_string(r as u32, c as u16, *value).unwrap();
        }
    }
    let bytes = book.save_to_buffer().unwrap();
    let mut project = empty_like(&example::house());
    let report = import_xlsx(&mut project, &bytes).unwrap();
    assert_eq!(report.circuits_added, 2);
    assert_eq!(report.errors.len(), 3, "{:?}", report.errors);
    assert!(report.errors[0].contains("fila 4") && report.errors[0].contains("mil"));
    assert!(report.errors[1].contains("Calefactor"));
    assert!(report.errors[2].contains("Motor hp"));
    let panel = &project.panels[0];
    assert_eq!(panel.circuits[0].load.kind, LoadKind::Lighting);
    assert_eq!(panel.circuits[0].load.power_w, 1500.0);
    assert_eq!(panel.circuits[1].load.kind, LoadKind::Motor);
    assert_eq!(panel.circuits[1].load.motor.as_ref().unwrap().hp, "7-1/2");
    assert!(report.summary().contains("2 circuitos nuevos"));
}

#[test]
fn a_sheet_without_the_key_columns_is_rejected() {
    let mut book = Workbook::new();
    let sheet = book.add_worksheet();
    sheet.write_string(0, 0, "Circuito").unwrap();
    let bytes = book.save_to_buffer().unwrap();
    let mut project = example::house();
    let error = import_xlsx(&mut project, &bytes).unwrap_err();
    assert!(error.contains("Tablero"), "{error}");
    assert!(import_xlsx(&mut project, b"no es excel").is_err());
}
