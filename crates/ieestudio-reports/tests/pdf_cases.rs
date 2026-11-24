//! La memoria se compone sin errores y contiene lo calculado.

use ieestudio_project::quick::{quick_circuit, QuickInput};
use ieestudio_project::{calculate, example};
use ieestudio_reports::{compose_project, project_pdf, project_source, quick_pdf};
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

/// Páginas de un PDF: los objetos `/Type/Page` (no `/Pages`).
fn pdf_pages(pdf: &[u8]) -> usize {
    pdf.windows(10)
        .enumerate()
        .filter(|(i, w)| *w == b"/Type/Page" && pdf.get(i + 10) != Some(&b's'))
        .count()
}

/// La vista previa compone una vez y convierte en imagen solo la página que
/// pide; lo que ve el usuario tiene las mismas páginas que el PDF.
#[test]
fn the_composed_report_has_the_pages_of_the_pdf() {
    let project = example::house();
    let result = calculate(&project).unwrap();
    let composed = compose_project(&project, &result).unwrap();
    let pdf = composed.pdf().unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(composed.page_count() >= 5);
    assert_eq!(composed.page_count(), pdf_pages(&pdf));
    // Exportar no cambia: el PDF de la composición es el de siempre.
    assert_eq!(
        pdf_pages(&project_pdf(&project, &result).unwrap()),
        composed.page_count()
    );
    // Carta vertical: 612 × 792 puntos.
    let (w, h) = composed.page_size_pt(0).unwrap();
    assert!((w - 612.0).abs() < 0.01 && (h - 792.0).abs() < 0.01);
    // Una página a 1 px/pt mide 612 × 792; a 2 px/pt, el doble.
    let page = composed.render_page(0, 1.0).unwrap();
    assert_eq!((page.width, page.height), (612, 792));
    assert_eq!(page.rgba.len(), 612 * 792 * 4);
    let sharp = composed
        .render_page(composed.page_count() - 1, 2.0)
        .unwrap();
    assert_eq!((sharp.width, sharp.height), (1224, 1584));
    // Fuera del documento no hay página.
    assert!(composed.render_page(composed.page_count(), 1.0).is_none());
    assert!(composed.page_size_pt(composed.page_count()).is_none());
}
