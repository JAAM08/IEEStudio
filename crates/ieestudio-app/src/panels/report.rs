//! Vista Memoria: la vista previa de la memoria de cálculo, páginas blancas
//! como se imprimirán, y su ficha con la firma, «Antes de entregar» y lo que
//! se puede exportar.
//!
//! La memoria se compone una vez por revisión del proyecto y solo se
//! convierten en imagen las páginas que se ven (y una de cada lado), con un
//! tope de [`MAX_TEXTURES`] imágenes en memoria sin importar cuántas páginas
//! tenga el documento.

use std::ops::Range;

use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular as icon;

use super::Shared;
use crate::guided::{problem_counts, Export};
use crate::state::Doc;
use crate::theme::{self, Palette};
use crate::widgets::{empty_state, muted, property, section, strong};

/// Imágenes de página que se guardan a la vez: unos 30 MB a 1.6 px/pt.
pub const MAX_TEXTURES: usize = 6;

/// Nitidez máxima de una página en pantalla, en píxeles por punto.
const MAX_PIXELS_PER_POINT: f32 = 2.0;

/// Páginas nuevas que se convierten por cuadro: así desplazarse no se traba.
const RENDERS_PER_FRAME: usize = 2;

/// Separación entre páginas en la vista previa.
const GAP: f32 = theme::SP4;

/// Una página ya convertida en imagen.
struct Cached {
    page: usize,
    pixels_per_point: f32,
    texture: egui::TextureHandle,
}

/// La memoria compuesta y las imágenes de las páginas a la vista.
#[derive(Default)]
pub struct Preview {
    composed: Option<ieestudio_reports::Composed>,
    /// Tamaño de cada página en puntos.
    sizes: Vec<Vec2>,
    cache: Vec<Cached>,
    revision: Option<u64>,
    error: Option<String>,
    /// Hay que componer: en un cuadro se avisa y en el siguiente se compone.
    pending: bool,
    announced: bool,
    pub zoom: f32,
    /// Página que está a la mitad de la vista, para la barra.
    current: usize,
}

impl Preview {
    /// Compone la memoria con los datos actuales. Es lo único caro: se pide
    /// al entrar a Memoria o con «Actualizar», nunca con cada tecla.
    fn compose(&mut self, doc: &Doc) {
        self.revision = Some(doc.revision);
        self.cache.clear();
        self.sizes.clear();
        self.error = None;
        self.composed = None;
        self.pending = false;
        self.announced = false;
        let composed = doc
            .result
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|r| ieestudio_reports::compose_project(&doc.project, r));
        match composed {
            Ok(composed) => {
                self.sizes = (0..composed.page_count())
                    .map(|i| {
                        let (w, h) = composed.page_size_pt(i).unwrap_or((612.0, 792.0));
                        Vec2::new(w as f32, h as f32)
                    })
                    .collect();
                self.composed = Some(composed);
            }
            Err(e) => self.error = Some(e),
        }
    }

    /// Al entrar a la vista: si el proyecto cambió desde la última vez, se
    /// vuelve a componer.
    pub fn entering(&mut self, revision: u64) {
        if self.revision.is_some_and(|r| r != revision) {
            self.pending = true;
        }
    }

    /// El PDF de lo ya compuesto, si corresponde a la versión `revision` del
    /// proyecto: así exportar desde Memoria no vuelve a componer.
    pub fn pdf(&self, revision: u64) -> Option<Result<Vec<u8>, String>> {
        let composed = self.composed.as_ref()?;
        (self.revision == Some(revision)).then(|| composed.pdf())
    }

    /// Asegura las imágenes de `wanted` a la nitidez pedida, convirtiendo a
    /// lo más [`RENDERS_PER_FRAME`] páginas, y suelta las que sobran.
    /// Devuelve `true` si quedó alguna por convertir.
    fn ensure(&mut self, ctx: &egui::Context, wanted: Range<usize>, pixels_per_point: f32) -> bool {
        let Some(composed) = &self.composed else {
            return false;
        };
        let mut rendered = 0;
        let mut missing = false;
        for page in wanted.clone() {
            let fresh = self.cache.iter().any(|c| {
                c.page == page && (c.pixels_per_point / pixels_per_point - 1.0).abs() < 0.25
            });
            if fresh {
                continue;
            }
            if rendered == RENDERS_PER_FRAME {
                missing = true;
                break;
            }
            let Some(image) = composed.render_page(page, f64::from(pixels_per_point)) else {
                continue;
            };
            let image = egui::ColorImage::from_rgba_premultiplied(
                [image.width as usize, image.height as usize],
                &image.rgba,
            );
            let texture = ctx.load_texture(
                format!("memoria-{page}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            self.cache.retain(|c| c.page != page);
            self.cache.push(Cached {
                page,
                pixels_per_point,
                texture,
            });
            rendered += 1;
        }
        let keep = keep_nearest(
            &self.cache.iter().map(|c| c.page).collect::<Vec<_>>(),
            &wanted,
            MAX_TEXTURES,
        );
        self.cache.retain(|c| keep.contains(&c.page));
        missing
    }
}

/// Dónde empieza cada página, de arriba abajo, con `gap` entre una y otra.
fn page_tops(heights: &[f32], gap: f32) -> Vec<f32> {
    let mut y = gap;
    heights
        .iter()
        .map(|h| {
            let top = y;
            y += h + gap;
            top
        })
        .collect()
}

/// Páginas que se ven entre `top` y `bottom`, más una de cada lado, para que
/// ya estén listas al desplazarse.
pub fn visible_pages(tops: &[f32], heights: &[f32], top: f32, bottom: f32) -> Range<usize> {
    let seen: Vec<usize> = (0..tops.len())
        .filter(|&i| tops[i] < bottom && tops[i] + heights[i] > top)
        .collect();
    match (seen.first(), seen.last()) {
        (Some(&first), Some(&last)) => first.saturating_sub(1)..(last + 2).min(tops.len()),
        _ => 0..0,
    }
}

/// Qué páginas conservar: todas las que se quieren y, si sobra lugar, las
/// más cercanas a ellas, hasta `max`.
pub fn keep_nearest(cached: &[usize], wanted: &Range<usize>, max: usize) -> Vec<usize> {
    let distance = |page: usize| {
        if wanted.contains(&page) {
            0
        } else if page < wanted.start {
            wanted.start - page
        } else {
            page + 1 - wanted.end
        }
    };
    let mut pages: Vec<usize> = cached.to_vec();
    pages.sort_by_key(|&p| (distance(p), p));
    pages.truncate(max);
    pages
}

/// La vista previa: barra con actualizar, exportar y zoom, y las páginas.
/// Devuelve `true` si se pidió exportar el PDF.
pub fn show(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) -> bool {
    let p = shared.palette;
    let preview = &mut shared.preview;
    if preview.zoom == 0.0 {
        // De entrada, la hoja carta a 75 % o al ancho de la ventana si no cabe.
        let room = ui.available_width() - ui.spacing().scroll.allocated_width();
        let fit = (room - 2.0 * GAP) / 612.0;
        preview.zoom = fit.clamp(0.3, 0.75);
    }
    if preview.revision.is_none() {
        preview.pending = true;
    }
    let stale = preview.revision.is_some_and(|r| r != doc.revision);
    let mut export = false;
    ui.horizontal_wrapped(|ui| {
        if crate::widgets::primary_bar_button(ui, &p, &format!("{}  Exportar PDF", icon::EXPORT))
            .on_hover_text("La memoria de cálculo completa, como se imprime")
            .clicked()
        {
            export = true;
        }
        if ui
            .button(format!("{} Actualizar", icon::ARROW_CLOCKWISE))
            .on_hover_text("Volver a componer la memoria con los datos actuales")
            .clicked()
        {
            preview.pending = true;
        }
        ui.separator();
        if ui.button(icon::MINUS).on_hover_text("Alejar").clicked() {
            preview.zoom = (preview.zoom / 1.2).max(0.3);
        }
        ui.label(RichText::new(format!("{:.0} %", preview.zoom * 100.0)).monospace());
        if ui.button(icon::PLUS).on_hover_text("Acercar").clicked() {
            preview.zoom = (preview.zoom * 1.2).min(2.5);
        }
        if !preview.sizes.is_empty() {
            ui.label(muted(
                &p,
                format!(
                    "Página {} de {} · {} {} en memoria",
                    preview.current + 1,
                    preview.sizes.len(),
                    preview.cache.len(),
                    if preview.cache.len() == 1 {
                        "imagen"
                    } else {
                        "imágenes"
                    }
                ),
            ));
        }
        if stale && !preview.pending {
            ui.label(
                RichText::new(format!(
                    "{} El proyecto cambió: actualiza la vista previa",
                    icon::WARNING_CIRCLE
                ))
                .color(p.warn),
            );
        }
    });
    ui.add_space(theme::SP2);
    // Las hojas blancas van sobre el tono del cromo, en los dos temas.
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, theme::RADIUS_CARD, p.bg);
    if preview.pending {
        // Se avisa en este cuadro y se compone en el siguiente: en el
        // navegador la composición ocupa el hilo principal.
        if preview.announced {
            preview.compose(doc);
        } else {
            preview.announced = true;
            ui.centered_and_justified(|ui| {
                ui.label(muted(
                    &p,
                    format!("{} Componiendo la memoria…", icon::HOURGLASS),
                ));
            });
            ui.ctx().request_repaint();
            return export;
        }
    }
    if let Some(e) = &preview.error {
        empty_state(ui, &p, icon::FILE_PDF, "No se pudo componer la memoria", e);
        return export;
    }
    let zoom = preview.zoom;
    let heights: Vec<f32> = preview.sizes.iter().map(|s| s.y * zoom).collect();
    let tops = page_tops(&heights, GAP);
    let total = tops
        .last()
        .zip(heights.last())
        .map_or(0.0, |(t, h)| t + h + GAP);
    let widest = preview.sizes.iter().map(|s| s.x * zoom).fold(0.0, f32::max);
    let pixels_per_point = (zoom * ui.ctx().pixels_per_point()).clamp(0.5, MAX_PIXELS_PER_POINT);
    // Ancho visible sin la barra de desplazamiento: ahí se centran las hojas.
    let room = ui.available_width() - ui.spacing().scroll.allocated_width();
    egui::ScrollArea::both()
        .id_salt("memoria")
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, viewport| {
            let width = room.max(widest + 2.0 * GAP);
            ui.set_min_size(Vec2::new(width, total));
            let origin = ui.min_rect().min;
            let wanted = visible_pages(&tops, &heights, viewport.top(), viewport.bottom());
            let middle = viewport.center().y;
            preview.current = (0..tops.len())
                .find(|&i| tops[i] + heights[i] + GAP / 2.0 >= middle)
                .unwrap_or(0);
            if preview.ensure(ui.ctx(), wanted.clone(), pixels_per_point) {
                ui.ctx().request_repaint();
            }
            for i in wanted {
                let size = preview.sizes[i] * zoom;
                // Cada hoja se centra en lo visible; la que no cabe (el
                // unifilar apaisado) empieza a la izquierda y se desplaza.
                let x = if size.x + 2.0 * GAP <= room {
                    (room - size.x) / 2.0
                } else {
                    GAP
                };
                let rect = egui::Rect::from_min_size(origin + Vec2::new(x, tops[i]), size);
                // La hoja flota sobre la mesa: la misma sombra suave de lo flotante.
                ui.painter().add(
                    egui::Shadow {
                        offset: [0, 2],
                        blur: 8,
                        spread: 0,
                        color: Color32::from_black_alpha(if p.dark { 110 } else { 40 }),
                    }
                    .as_shape(rect, 0),
                );
                match preview.cache.iter().find(|c| c.page == i) {
                    Some(cached) => {
                        ui.painter().image(
                            cached.texture.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    None => {
                        ui.painter().rect_filled(rect, 0.0, Color32::WHITE);
                    }
                }
            }
        });
    export
}

/// La vista Memoria completa: la vista previa y su ficha. En ventanas anchas
/// la ficha va a la derecha; en angostas, arriba y plegable.
pub fn view(ui: &mut egui::Ui, doc: &mut Doc, shared: &mut Shared) -> Option<Export> {
    let p = shared.palette;
    let mut request = None;
    let wide = ui.available_width() >= crate::layout::WIDE;
    if wide {
        egui::Panel::right("memoria-ficha")
            .resizable(true)
            .default_size(crate::layout::INSPECTOR_W)
            .size_range(280.0..=480.0)
            .frame(
                egui::Frame::new()
                    .fill(p.panel)
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("memoria-ficha")
                    .auto_shrink([false, false])
                    .show(ui, |ui| request = ficha(ui, &p, doc, shared));
            });
    } else {
        egui::Panel::top("memoria-ficha-angosta")
            .frame(
                egui::Frame::new()
                    .fill(p.panel)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ui, |ui| {
                egui::CollapsingHeader::new(strong("Firma y entrega"))
                    .id_salt("memoria-ficha-plegable")
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("memoria-ficha")
                            .max_height(ui.ctx().content_rect().height() * 0.5)
                            .show(ui, |ui| request = ficha(ui, &p, doc, shared));
                    });
            });
    }
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .inner_margin(egui::Margin::symmetric(16, 12)),
        )
        .show(ui, |ui| {
            if show(ui, doc, shared) {
                request = Some(Export::Pdf);
            }
        });
    request
}

/// Ficha de la memoria: firma, «Antes de entregar» y otras exportaciones.
fn ficha(ui: &mut egui::Ui, p: &Palette, doc: &mut Doc, shared: &mut Shared) -> Option<Export> {
    let mut request = None;
    ui.label(strong("Memoria de cálculo").size(theme::HEAD));
    ui.label(
        muted(
            p,
            "Los datos de firma y lo que entregas. Lo demás ya salió del proyecto.",
        )
        .size(theme::SECONDARY),
    );

    section(ui, p, "Firma");
    let mut changed = false;
    let info = &mut doc.project.info;
    for (label, value, hint) in [
        ("Responsable", &mut info.responsible, "Nombre del ingeniero"),
        ("Cédula", &mut info.license, "Número de cédula"),
        ("Revisión", &mut info.revision, "A, 0, 1…"),
        ("Fecha", &mut info.date, "Vacía: la del día"),
    ] {
        property(ui, p, label, |ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(value)
                        .hint_text(hint)
                        .desired_width(f32::INFINITY),
                )
                .changed();
        });
    }
    crate::widgets::end_group(ui);
    if changed {
        doc.edited();
    }

    // Antes de entregar: lo que conviene revisar, cada punto con su estado.
    let (fails, pending) = problem_counts(doc);
    let complete = doc
        .result
        .as_ref()
        .is_ok_and(|r| r.verdict.calculation == ieestudio_project::verdict::Calculation::Complete);
    let info = &doc.project.info;
    let signed = !info.responsible.trim().is_empty() && !info.license.trim().is_empty();
    let mut items = vec![
        check(
            p,
            complete,
            false,
            "Todos los tramos calculados",
            "Hay tramos sin calcular",
        ),
        check(
            p,
            fails == 0,
            true,
            "Nada incumple",
            &match fails {
                1 => "1 verificación no cumple".to_string(),
                n => format!("{n} verificaciones no cumplen"),
            },
        ),
        check(
            p,
            pending == 0,
            false,
            "Nada quedó sin evaluar",
            &match pending {
                1 => "1 verificación sin evaluar".to_string(),
                n => format!("{n} verificaciones sin evaluar"),
            },
        ),
        check(
            p,
            signed,
            false,
            "Responsable y cédula capturados",
            "Faltan el responsable o su cédula",
        ),
    ];
    if let Ok(r) = &doc.result {
        for o in &r.verdict.out_of_scope {
            items.push((
                icon::INFO,
                p.ink2,
                format!("{}: fuera de alcance, se anota en la memoria", o.name),
            ));
        }
    }
    list_group(ui, p, "Antes de entregar", &items);

    section(ui, p, "También puedes exportar");
    crate::widgets::end_group(ui);
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(format!("{}  Cuadro de cargas en Excel", icon::FILE_XLS))
            .clicked()
        {
            request = Some(Export::Excel);
        }
        if ui
            .button(format!("{}  Unifilar en DXF", icon::FLOW_ARROW))
            .clicked()
        {
            request = Some(Export::Dxf);
        }
        if ui
            .button(format!("{}  Ver el unifilar", icon::ARROWS_OUT))
            .on_hover_text("Como sale en la memoria: blanco y negro, con su hoja")
            .clicked()
        {
            shared.diagram_full = true;
            shared.diagram_fit = true;
            shared.diagram_mode = ieestudio_diagrams::Mode::Plan;
        }
    });
    request
}

/// Un punto de «Antes de entregar»: lo que no cumple en rojo, lo que falta en
/// naranja, lo resuelto en verde; siempre con ícono y texto.
fn check(
    p: &Palette,
    ok: bool,
    failing: bool,
    done: &str,
    missing: &str,
) -> (&'static str, Color32, String) {
    match (ok, failing) {
        (true, _) => (icon::CHECK_CIRCLE, p.ok, done.to_string()),
        (false, true) => (icon::X_CIRCLE, p.bad, missing.to_string()),
        (false, false) => (icon::CIRCLE_DASHED, p.warn, missing.to_string()),
    }
}

/// Grupo de renglones de solo lectura (ícono y texto), con divisores finos.
fn list_group(ui: &mut egui::Ui, p: &Palette, title: &str, rows: &[(&str, Color32, String)]) {
    section(ui, p, title);
    crate::widgets::end_group(ui);
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(theme::RADIUS_CARD)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            for (k, (glyph, color, text)) in rows.iter().enumerate() {
                if k > 0 {
                    let x = ui.max_rect().x_range();
                    ui.painter().hline(
                        (x.min + theme::SP3)..=x.max,
                        ui.cursor().top(),
                        egui::Stroke::new(1.0, p.hair),
                    );
                }
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .show(ui, |ui| {
                        ui.set_min_height(theme::ROW_H - theme::SP2);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = theme::SP2;
                            ui.label(RichText::new(*glyph).size(theme::ICON).color(*color));
                            ui.label(RichText::new(text).color(if *color == p.ok {
                                p.ink
                            } else {
                                *color
                            }));
                        });
                    });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letter(n: usize, zoom: f32) -> (Vec<f32>, Vec<f32>) {
        let heights = vec![792.0 * zoom; n];
        (page_tops(&heights, GAP), heights)
    }

    #[test]
    fn only_the_pages_in_view_and_one_on_each_side_are_wanted() {
        let (tops, heights) = letter(522, 1.0);
        // Al principio: la primera página y la que sigue.
        assert_eq!(visible_pages(&tops, &heights, 0.0, 700.0), 0..2);
        // A la mitad del documento: la que se ve, más una de cada lado.
        let top = tops[200] + 10.0;
        let range = visible_pages(&tops, &heights, top, top + 700.0);
        assert_eq!(range, 199..202);
        // Al final no se pasa del documento.
        let last = tops[521];
        assert_eq!(visible_pages(&tops, &heights, last, last + 700.0), 520..522);
        // Alejado, caben varias a la vez.
        let (tops, heights) = letter(22, 0.3);
        let range = visible_pages(&tops, &heights, 0.0, 900.0);
        assert!(range.len() >= 3 && range.start == 0);
        // Sin páginas, nada.
        assert_eq!(visible_pages(&[], &[], 0.0, 900.0), 0..0);
    }

    #[test]
    fn at_most_six_images_are_kept_and_the_farthest_go_first() {
        let cached: Vec<usize> = (0..20).collect();
        let keep = keep_nearest(&cached, &(10..13), MAX_TEXTURES);
        assert_eq!(keep.len(), MAX_TEXTURES);
        for page in 10..13 {
            assert!(keep.contains(&page));
        }
        // Las que se quedan son las vecinas, no las del principio.
        assert!(keep.iter().all(|p| (8..15).contains(p)));
        // Con pocas, se quedan todas.
        assert_eq!(keep_nearest(&[3, 4], &(3..5), MAX_TEXTURES), vec![3, 4]);
    }

    #[test]
    fn pages_are_stacked_with_a_gap() {
        let tops = page_tops(&[100.0, 200.0, 50.0], 10.0);
        assert_eq!(tops, vec![10.0, 120.0, 330.0]);
    }
}
