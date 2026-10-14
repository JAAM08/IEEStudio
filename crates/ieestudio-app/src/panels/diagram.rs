//! Vista del diagrama unifilar.

use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Sense, Shape, Stroke, Vec2};
use ieestudio_diagrams::{Anchor, Diagram, Element, Font, Ink};

use crate::theme::{self, Palette};

fn color(p: &Palette, ink: Ink) -> Color32 {
    match ink {
        Ink::Text => p.ink,
        Ink::Muted => p.ink2,
        Ink::Accent => p.accent_text,
        Ink::Complies => p.ok,
        Ink::Fails => p.bad,
        Ink::NotEvaluated => p.warn,
    }
}

/// Barra de zoom y el diagrama, desplazable en ambos sentidos.
pub fn show(
    ui: &mut egui::Ui,
    p: &Palette,
    diagram: &Diagram,
    zoom: &mut f32,
    selected: Option<(usize, Option<usize>)>,
) -> Option<(usize, Option<usize>)> {
    let available = ui.available_width() - 16.0;
    ui.horizontal(|ui| {
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
    egui::ScrollArea::both()
        .auto_shrink(false)
        .show(ui, |ui| paint(ui, p, diagram, *zoom, selected))
        .inner
}

fn paint(
    ui: &mut egui::Ui,
    p: &Palette,
    diagram: &Diagram,
    zoom: f32,
    selected: Option<(usize, Option<usize>)>,
) -> Option<(usize, Option<usize>)> {
    let size = Vec2::new(diagram.width as f32, diagram.height as f32) * zoom;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 8.0, p.panel);
    let map = |(x, y): (f64, f64)| rect.min + Vec2::new(x as f32, y as f32) * zoom;
    let area = |h: &ieestudio_diagrams::Hotspot| {
        egui::Rect::from_min_size(
            map(h.min),
            Vec2::new(h.size.0 as f32, h.size.1 as f32) * zoom,
        )
    };
    // Resalte del elemento seleccionado y del que está bajo el cursor.
    let hover = response.hover_pos();
    for h in &diagram.hotspots {
        let r = area(h);
        if Some((h.panel, h.circuit)) == selected {
            painter.rect_filled(r, 4.0, p.selected);
            painter.rect_stroke(r, 4.0, Stroke::new(1.0, p.accent), egui::StrokeKind::Inside);
        } else if hover.is_some_and(|pos| r.contains(pos))
            && !(h.circuit.is_none()
                && diagram.hotspots.iter().any(|o| {
                    o.panel == h.panel && o.circuit.is_some() && area(o).contains(hover.unwrap())
                }))
        {
            painter.rect_filled(r, 4.0, p.raise.gamma_multiply(0.6));
        }
    }
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
    // Clic: el circuito tiene prioridad sobre la zona del tablero que lo contiene.
    let pos = response
        .interact_pointer_pos()
        .filter(|_| response.clicked())?;
    let hit = |circuit: bool| {
        diagram
            .hotspots
            .iter()
            .find(|h| h.circuit.is_some() == circuit && area(h).contains(pos))
    };
    hit(true)
        .or_else(|| hit(false))
        .map(|h| (h.panel, h.circuit))
}
