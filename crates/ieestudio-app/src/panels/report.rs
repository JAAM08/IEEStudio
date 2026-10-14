//! Vista previa de la memoria: páginas blancas, como se imprimirán, dentro
//! del entorno oscuro.

use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular as icon;

use super::Shared;
use crate::state::Doc;
use crate::widgets::{empty_state, muted};

/// Páginas ya rasterizadas, con la revisión del proyecto de la que salieron.
#[derive(Default)]
pub struct Preview {
    pages: Vec<egui::TextureHandle>,
    revision: Option<u64>,
    error: Option<String>,
    pub zoom: f32,
}

/// Puntos por pulgada de la rasterización, respecto a 72.
const PIXELS_PER_POINT: f64 = 1.6;

impl Preview {
    fn refresh(&mut self, ctx: &egui::Context, doc: &Doc) {
        self.revision = Some(doc.revision);
        self.pages.clear();
        self.error = None;
        let pages = doc
            .result
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|r| ieestudio_reports::project_pages(&doc.project, r, PIXELS_PER_POINT));
        match pages {
            Ok(pages) => {
                for (i, page) in pages.into_iter().enumerate() {
                    let image = egui::ColorImage::from_rgba_premultiplied(
                        [page.width as usize, page.height as usize],
                        &page.rgba,
                    );
                    self.pages.push(ctx.load_texture(
                        format!("memoria-{i}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            Err(e) => self.error = Some(e),
        }
    }
}

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) -> bool {
    let p = shared.palette;
    let preview = &mut shared.preview;
    if preview.zoom == 0.0 {
        preview.zoom = 0.75;
    }
    let stale = preview.revision != Some(doc.revision);
    if preview.revision.is_none() {
        preview.refresh(ui.ctx(), doc);
    }
    let mut export = false;
    ui.horizontal(|ui| {
        if ui
            .button(format!("{} Actualizar", icon::ARROW_CLOCKWISE))
            .on_hover_text("Volver a componer la memoria con los datos actuales")
            .clicked()
        {
            preview.refresh(ui.ctx(), doc);
        }
        if ui
            .button(format!("{} Exportar PDF…", icon::EXPORT))
            .clicked()
        {
            export = true;
        }
        ui.separator();
        if ui.button(icon::MINUS).clicked() {
            preview.zoom = (preview.zoom / 1.2).max(0.3);
        }
        ui.label(RichText::new(format!("{:.0} %", preview.zoom * 100.0)).monospace());
        if ui.button(icon::PLUS).clicked() {
            preview.zoom = (preview.zoom * 1.2).min(2.5);
        }
        if !preview.pages.is_empty() {
            ui.label(muted(
                &p,
                format!("{} páginas, tamaño carta", preview.pages.len()),
            ));
        }
        if stale && preview.revision.is_some() {
            ui.label(
                RichText::new(format!(
                    "{} El proyecto cambió: actualiza la vista previa",
                    icon::WARNING_CIRCLE
                ))
                .color(p.warn),
            );
        }
    });
    ui.separator();
    if let Some(e) = &preview.error {
        empty_state(ui, &p, icon::FILE_PDF, "No se pudo componer la memoria", e);
        return export;
    }
    let zoom = preview.zoom;
    // Mesa de trabajo más oscura que la hoja, como en Acrobat, en los dos temas.
    let canvas = if p.dark {
        p.bg
    } else {
        Color32::from_rgb(0xc4, 0xc9, 0xcf)
    };
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, 0.0, canvas);
    egui::ScrollArea::both()
        .id_salt("memoria")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                for texture in &preview.pages {
                    // Tamaño en puntos de pantalla: la página carta mide 612 × 792 pt.
                    let size = texture.size_vec2() / PIXELS_PER_POINT as f32 * zoom;
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    ui.painter().rect_filled(
                        rect.translate(Vec2::new(3.0, 4.0)),
                        2.0,
                        Color32::from_black_alpha(90),
                    );
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.add_space(16.0);
                }
            });
        });
    export
}
