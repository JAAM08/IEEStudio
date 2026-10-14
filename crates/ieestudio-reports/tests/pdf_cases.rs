//! La memoria se compone sin errores y contiene lo calculado.

use ieestudio_project::quick::{quick_circuit, QuickInput};
use ieestudio_project::{calculate, example};
use ieestudio_reports::{project_pages, project_pdf, project_source, quick_pdf};
use ieestudio_rules::Profile;

#[test]
fn project_report_renders() {
    let project = example::house();
    let result = calculate(&project).unwrap();
    let source = project_source(&project, &result);
    // El procedimiento de C-02 viaja tal cual a la memoria.
    assert!(source.contains("I = 1,800 / (127 · 0.90) = 15.75 A"));
    let pdf = project_pdf(&project, &result).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    if let Ok(path) = std::env::var("IEESTUDIO_PDF_OUT") {
        std::fs::write(format!("{path}/memoria.pdf"), &pdf).unwrap();
    }
}

#[test]
fn quick_sheet_renders() {
    let input = QuickInput::default();
    let result = quick_circuit(Profile::nom_001_sede_2012(), &input).unwrap();
    let pdf = quick_pdf(&input, &result).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    if let Ok(path) = std::env::var("IEESTUDIO_PDF_OUT") {
        std::fs::write(format!("{path}/hoja.pdf"), &pdf).unwrap();
    }
}

#[test]
fn project_pages_render_for_preview() {
    let project = example::house();
    let result = calculate(&project).unwrap();
    let pages = project_pages(&project, &result, 1.0).unwrap();
    assert!(pages.len() >= 5);
    // Carta vertical: 612 × 792 puntos.
    assert_eq!((pages[0].width, pages[0].height), (612, 792));
    assert_eq!(pages[0].rgba.len(), 612 * 792 * 4);
}
