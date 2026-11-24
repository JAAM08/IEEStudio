//! Vista del diagrama unifilar.

use eframe::egui::{self, Align2, Color32, FontFamily, FontId, Sense, Shape, Stroke, Vec2};
use ieestudio_diagrams::{Anchor, Diagram, Element, Font, Ink, Mode};

use egui_phosphor::regular as icon;

use crate::shell::CAPSULE;
use crate::theme::{self, Palette};

fn color(p: &Palette, ink: Ink, paper: bool) -> Color32 {
    if paper {
        // Como se imprime: tinta oscura sobre papel blanco.
        return match ink {
            Ink::Muted => Color32::from_rgb(0x5d, 0x68, 0x77),
            Ink::Accent => Color32::from_rgb(0xb3, 0x26, 0x1e),
            _ => Color32::from_rgb(0x1b, 0x22, 0x2c),
        };
    }
    match ink {
        Ink::Text => p.ink,
        Ink::Muted => p.ink2,
        Ink::Accent => p.accent_text,
        Ink::Complies => p.ok,
        Ink::Fails => p.bad,
        Ink::NotEvaluated => p.warn,
    }
}

/// Zoom mínimo y máximo del unifilar.
pub const ZOOM_RANGE: (f32, f32) = (0.2, 3.0);

/// Lo que se recuerda de un cuadro al siguiente para acercar sin que se mueva
/// lo que está bajo el cursor.
#[derive(Clone, Copy)]
struct Viewport {
    /// Esquina del dibujo en pantalla.
    origin: egui::Pos2,
    offset: Vec2,
    /// Lo visible del área desplazable.
    inner: egui::Rect,
    /// Desplazamiento que falta aplicar: al acercar, el área todavía no sabe
    /// que el dibujo creció y lo recorta un cuadro.
    pending: Option<Vec2>,
}

/// Desplazamiento que deja el punto del dibujo que está bajo `pointer` en el
/// mismo lugar de la pantalla al pasar de `zoom` a `new_zoom`.
pub fn anchored_offset(
    origin: egui::Pos2,
    offset: Vec2,
    pointer: egui::Pos2,
    zoom: f32,
    new_zoom: f32,
) -> Vec2 {
    let point = (pointer - origin) / zoom;
    let new_origin = pointer - point * new_zoom;
    (offset + (origin - new_origin)).max(Vec2::ZERO)
}

/// El diagrama, desplazable en ambos sentidos, con sus controles flotando en
/// una cápsula arriba a la derecha. `mode` elige entre la vista de revisión,
/// con colores por estado, y la de plano, en blanco y negro con su hoja; el
/// cambio se ve en el siguiente cuadro.
///
/// Se acerca con la rueda y Ctrl (o Cmd), o pellizcando, sobre el punto del
/// cursor; se recorre arrastrando o con la rueda.
pub fn show(
    ui: &mut egui::Ui,
    p: &Palette,
    diagram: &Diagram,
    zoom: &mut f32,
    mode: &mut Mode,
    selected: Option<(usize, Option<usize>)>,
) -> Option<(usize, Option<usize>)> {
    let id = ui.id().with("unifilar");
    let last: Option<Viewport> = ui.data(|d| d.get_temp(id));
    let mut scroll = egui::ScrollArea::both()
        .id_salt("unifilar")
        .auto_shrink(false)
        .scroll_source(egui::scroll_area::ScrollSource::ALL);
    let mut pending = None;
    if let Some(last) = last {
        let (delta, pointer) = ui.input(|i| (i.zoom_delta(), i.pointer.hover_pos()));
        let pointer = pointer.filter(|&at| last.inner.contains(at));
        if let (true, Some(at)) = (delta != 1.0, pointer) {
            let new_zoom = (*zoom * delta).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
            if new_zoom != *zoom {
                pending = Some(anchored_offset(
                    last.origin,
                    last.offset,
                    at,
                    *zoom,
                    new_zoom,
                ));
                *zoom = new_zoom;
            }
        } else {
            pending = last.pending;
        }
    }
    if let Some(offset) = pending {
        scroll = scroll.scroll_offset(offset);
    }
    let paper = *mode == Mode::Plan;
    let mut origin = egui::Pos2::ZERO;
    let output = scroll.show(ui, |ui| {
        origin = ui.cursor().min;
        paint(ui, p, diagram, *zoom, paper, selected)
    });
    let inner = output.inner_rect;
    ui.data_mut(|d| {
        d.insert_temp(
            id,
            Viewport {
                origin,
                offset: output.state.offset,
                inner,
                // Se aplica un cuadro más, ya con el tamaño nuevo del dibujo.
                pending: pending.filter(|o| (output.state.offset - *o).length() > 0.5),
            },
        )
    });
    if pending.is_some() {
        ui.ctx().request_repaint();
    }
    controls(ui, p, id, inner, diagram, zoom, mode);
    output.inner
}

/// Los controles del unifilar en una cápsula flotante, arriba a la derecha:
/// revisión o plano, alejar, zoom, acercar y ajustar al ancho.
fn controls(
    ui: &egui::Ui,
    p: &Palette,
    id: egui::Id,
    inner: egui::Rect,
    diagram: &Diagram,
    zoom: &mut f32,
    mode: &mut Mode,
) {
    let anchor = inner.right_top() + Vec2::new(-theme::SP3, theme::SP3);
    egui::Area::new(id.with("controles"))
        .order(egui::Order::Middle)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(anchor)
        .constrain_to(inner)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(p.raise)
                .stroke(Stroke::new(1.0, p.line))
                .corner_radius(CAPSULE)
                .shadow(ui.style().visuals.popup_shadow)
                .inner_margin(egui::Margin::symmetric(4, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = theme::SP1;
                        crate::widgets::segmented(
                            ui,
                            p,
                            mode,
                            &[
                                (
                                    Mode::Review,
                                    "Revisión",
                                    "Colores según cumple, no cumple o sin evaluar.",
                                ),
                                (
                                    Mode::Plan,
                                    "Plano",
                                    "Como sale en la memoria y en el DXF: blanco y negro, con simbología, notas y cajetín.",
                                ),
                            ],
                        );
                        let round = |ui: &mut egui::Ui, glyph: &str, tip: &str| {
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new(glyph).size(theme::ICON_BAR),
                                )
                                .frame(false)
                                .min_size(Vec2::splat(theme::CONTROL_H)),
                            )
                            .on_hover_text(tip)
                            .clicked()
                        };
                        if round(ui, icon::MINUS, "Alejar (Ctrl y la rueda, o pellizcar)") {
                            *zoom = (*zoom / 1.2).max(ZOOM_RANGE.0);
                        }
                        ui.label(
                            egui::RichText::new(format!("{:.0} %", *zoom * 100.0))
                                .monospace()
                                .color(p.ink2),
                        );
                        if round(ui, icon::PLUS, "Acercar (Ctrl y la rueda, o pellizcar)") {
                            *zoom = (*zoom * 1.2).min(ZOOM_RANGE.1);
                        }
                        if round(ui, icon::ARROWS_OUT_LINE_HORIZONTAL, "Ajustar al ancho") {
                            *zoom = ((inner.width() - theme::SP4) / diagram.width as f32)
                                .clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
                        }
                    });
                });
        });
}

pub fn paint(
    ui: &mut egui::Ui,
    p: &Palette,
    diagram: &Diagram,
    zoom: f32,
    paper: bool,
    selected: Option<(usize, Option<usize>)>,
) -> Option<(usize, Option<usize>)> {
    let size = Vec2::new(diagram.width as f32, diagram.height as f32) * zoom;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        if paper { 0.0 } else { 8.0 },
        if paper { Color32::WHITE } else { p.panel },
    );
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
    // Solo se dibuja lo que se ve: con cientos de circuitos, acomodar el
    // texto de lo que queda fuera es lo que hacía lento el cuadro.
    let clip = painter.clip_rect().expand(2.0);
    for element in &diagram.elements {
        if !clip.intersects(bounds(element, zoom, map)) {
            continue;
        }
        match element {
            Element::Path {
                points,
                width,
                ink,
                dashed,
            } => {
                let points: Vec<egui::Pos2> = points.iter().map(|&pt| map(pt)).collect();
                let stroke = Stroke::new(*width as f32 * zoom, color(p, *ink, paper));
                if *dashed {
                    painter.extend(Shape::dashed_line(&points, stroke, 6.0 * zoom, 4.0 * zoom));
                } else {
                    painter.add(Shape::line(points, stroke));
                }
            }
            Element::Rect {
                min,
                size,
                ink,
                dashed,
            } => {
                let r = egui::Rect::from_min_size(
                    map(*min),
                    Vec2::new(size.0 as f32, size.1 as f32) * zoom,
                );
                let stroke = Stroke::new(1.2 * zoom, color(p, *ink, paper));
                if *dashed {
                    let corners = vec![
                        r.left_top(),
                        r.right_top(),
                        r.right_bottom(),
                        r.left_bottom(),
                        r.left_top(),
                    ];
                    painter.extend(Shape::dashed_line(&corners, stroke, 6.0 * zoom, 4.0 * zoom));
                } else {
                    painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
                }
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
                    color(p, *ink, paper),
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

/// La vista Unifilar en escritorio y tableta: título, «Exportar DXF» y el
/// dibujo. Elegir un tablero o circuito lo selecciona; su ficha se ve en el
/// inspector o en su cajón.
pub fn view(
    ui: &mut egui::Ui,
    doc: &mut crate::state::Doc,
    shared: &mut super::Shared,
) -> Option<crate::views::Export> {
    use crate::state::Selection;
    let p = shared.palette;
    let mut request = None;
    egui::Panel::top("unifilar-completo-barra")
        .frame(
            egui::Frame::new()
                .fill(p.bg)
                .inner_margin(egui::Margin::symmetric(16, 8)),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(crate::widgets::strong("Diagrama unifilar").size(theme::HEAD));
                // En ventanas angostas la indicación no cabe junto al botón.
                if ui.available_width() > 600.0 {
                    ui.label(
                        crate::widgets::muted(
                            &p,
                            "Haz clic en un tablero o circuito para ver su ficha.",
                        )
                        .size(theme::SECONDARY),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::widgets::primary_bar_button(
                        ui,
                        &p,
                        &format!("{}  Exportar DXF", icon::FLOW_ARROW),
                    )
                    .clicked()
                    {
                        request = Some(crate::views::Export::Dxf);
                    }
                });
            });
        });
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.bg)
                .inner_margin(egui::Margin::symmetric(16, 4)),
        )
        .show(ui, |ui| {
            let Some(diagram) = shared.diagram(doc, shared.diagram_mode) else {
                crate::widgets::empty_state(
                    ui,
                    &p,
                    icon::WARNING_CIRCLE,
                    "No se pudo calcular el proyecto",
                    &doc.calc_error().unwrap_or_default(),
                );
                return;
            };
            if shared.diagram_fit {
                // Al ancho de la ventana: así se lee, y lo que no cabe a lo
                // alto se recorre. Sin agrandarlo de más en pantallas grandes.
                let fit = (ui.available_width() - theme::SP6) / diagram.width as f32;
                shared.diagram_zoom = fit.clamp(0.4, 1.3);
                shared.diagram_fit = false;
            }
            let selected = match doc.selection {
                Selection::Feeder(i) | Selection::Secondary(i) => Some((i, None)),
                Selection::Circuit(i, c) => Some((i, Some(c))),
                Selection::Project => None,
            };
            let before = shared.diagram_mode;
            if let Some(hit) = show(
                ui,
                &p,
                &diagram,
                &mut shared.diagram_zoom,
                &mut shared.diagram_mode,
                selected,
            ) {
                doc.selection = match hit {
                    (i, Some(c)) => Selection::Circuit(i, c),
                    (i, None) => Selection::Feeder(i),
                };
            }
            // La hoja del plano tiene otra proporción: se vuelve a ajustar.
            if before != shared.diagram_mode {
                shared.diagram_fit = true;
            }
        });
    request
}

/// Rectángulo en pantalla que ocupa un elemento, con holgura: el del texto se
/// estima por su número de letras, sin acomodarlo.
fn bounds(element: &Element, zoom: f32, map: impl Fn((f64, f64)) -> egui::Pos2) -> egui::Rect {
    match element {
        Element::Path { points, width, .. } => {
            let rect =
                egui::Rect::from_points(&points.iter().map(|&pt| map(pt)).collect::<Vec<_>>());
            rect.expand(*width as f32 * zoom)
        }
        Element::Rect { min, size, .. } => {
            egui::Rect::from_min_size(map(*min), Vec2::new(size.0 as f32, size.1 as f32) * zoom)
                .expand(2.0 * zoom)
        }
        Element::Text {
            pos,
            text,
            size,
            anchor,
            ..
        } => {
            let height = *size as f32 * zoom;
            let width = text.chars().count() as f32 * height * 0.65;
            let at = map(*pos);
            let left = match anchor {
                Anchor::Start => at.x,
                Anchor::Middle => at.x - width / 2.0,
                Anchor::End => at.x - width,
            };
            egui::Rect::from_min_max(
                egui::pos2(left, at.y - height * 1.2),
                egui::pos2(left + width, at.y + height * 0.5),
            )
            .expand(height * 0.5)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zooming_keeps_the_point_under_the_cursor_in_place() {
        let origin = egui::pos2(100.0, 50.0);
        let pointer = egui::pos2(400.0, 300.0);
        let offset = Vec2::new(200.0, 80.0);
        // El punto del dibujo bajo el cursor, antes y después de acercar.
        let point = (pointer - origin) / 1.0;
        let new = anchored_offset(origin, offset, pointer, 1.0, 2.0);
        let new_origin = origin - (new - offset);
        assert!(((pointer - new_origin) / 2.0 - point).length() < 1e-3);
        // Alejar hasta el principio no deja desplazamientos negativos.
        let back = anchored_offset(egui::pos2(0.0, 0.0), Vec2::ZERO, pointer, 2.0, 0.5);
        assert!(back.x >= 0.0 && back.y >= 0.0);
    }

    #[test]
    fn what_is_out_of_view_is_not_drawn() {
        let map = |(x, y): (f64, f64)| egui::pos2(x as f32, y as f32);
        let text = Element::Text {
            pos: (1000.0, 1000.0),
            text: "C-01 Alumbrado".to_string(),
            size: 10.0,
            font: Font::Regular,
            ink: Ink::Text,
            anchor: Anchor::Start,
        };
        let view = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::splat(800.0));
        assert!(!view.intersects(bounds(&text, 1.0, map)));
        // Un texto que empieza fuera pero entra en la vista sí se dibuja.
        let long = Element::Text {
            pos: (795.0, 400.0),
            text: "Alimentador general".to_string(),
            size: 10.0,
            font: Font::Regular,
            ink: Ink::Text,
            anchor: Anchor::End,
        };
        assert!(view.intersects(bounds(&long, 1.0, map)));
        let path = Element::Path {
            points: vec![(-50.0, 400.0), (900.0, 400.0)],
            width: 1.0,
            ink: Ink::Text,
            dashed: false,
        };
        assert!(view.intersects(bounds(&path, 1.0, map)));
    }
}
