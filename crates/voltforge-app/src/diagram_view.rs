//! Vista del diagrama unifilar.

use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Sense, Shape, Stroke, Vec2};
use voltforge_diagrams::{Anchor, Diagram, Element, Font, Ink};

use crate::theme::{self, Palette};

fn color(p: &Palette, ink: Ink) -> Color32 {
    match ink {
        Ink::Text => p.ink,
        Ink::Muted => p.ink2,
        Ink::Accent => p.accent,
        Ink::Complies => p.ok,
        Ink::Fails => p.bad,
        Ink::NotEvaluated => p.na,
    }
}

/// Barra de zoom y el diagrama, desplazable en ambos sentidos.
pub fn show(ui: &mut egui::Ui, p: &Palette, diagram: &Diagram, zoom: &mut f32) {
    let available = ui.available_width() - 16.0;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Diagrama unifilar")
                .family(theme::semibold())
                .size(18.0),
        );
        ui.add_space(12.0);
        if ui.button("−").clicked() {
            *zoom = (*zoom / 1.2).max(0.3);
        }
        ui.label(egui::RichText::new(format!("{:.0} %", *zoom * 100.0)).monospace());
        if ui.button("+").clicked() {
            *zoom = (*zoom * 1.2).min(3.0);
        }
        if ui.button("Ajustar al ancho").clicked() {
            *zoom = (available / diagram.width as f32).clamp(0.3, 3.0);
        }
        ui.label(
            egui::RichText::new(
                "Se dibuja solo a partir del proyecto; se actualiza con cada cambio.",
            )
            .color(p.ink2),
        );
    });
    ui.add_space(8.0);
    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
        paint(ui, p, diagram, *zoom);
    });
}

fn paint(ui: &mut egui::Ui, p: &Palette, diagram: &Diagram, zoom: f32) {
    let size = Vec2::new(diagram.width as f32, diagram.height as f32) * zoom;
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 8.0, p.panel);
    let map = |(x, y): (f64, f64)| rect.min + Vec2::new(x as f32, y as f32) * zoom;
    for element in &diagram.elements {
        match element {
            Element::Path { points, width, ink } => {
                painter.add(Shape::line(
                    points.iter().map(|&pt| map(pt)).collect(),
                    Stroke::new(*width as f32 * zoom, color(p, *ink)),
                ));
            }
            Element::Rect { min, size, ink } => {
                let r = egui::Rect::from_min_size(
                    map(*min),
                    Vec2::new(size.0 as f32, size.1 as f32) * zoom,
                );
                painter.rect_stroke(
                    r,
                    3.0 * zoom,
                    Stroke::new(1.4 * zoom, color(p, *ink)),
                    egui::StrokeKind::Middle,
                );
            }
            Element::Text {
                pos,
                text,
                size,
                font,
                ink,
                anchor,
            } => {
                let family = match font {
                    Font::Regular => FontFamily::Proportional,
                    Font::Bold => theme::semibold(),
                    Font::Mono => FontFamily::Monospace,
                };
                let align = match anchor {
                    Anchor::Start => Align2::LEFT_BOTTOM,
                    Anchor::Middle => Align2::CENTER_BOTTOM,
                    Anchor::End => Align2::RIGHT_BOTTOM,
                };
                // La posición es la línea base; se baja un poco por los descendentes.
                let at = map(*pos) + Vec2::new(0.0, 2.5 * zoom);
                painter.text(
                    at,
                    align,
                    text,
                    FontId::new(*size as f32 * zoom, family),
                    color(p, *ink),
                );
            }
        }
    }
}
