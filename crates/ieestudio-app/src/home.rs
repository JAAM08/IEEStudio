//! Pantalla de inicio: dos experiencias independientes, «Proyecto completo»
//! y «Cálculo rápido» (requisito del 18 de noviembre de 2026).

use std::path::PathBuf;

use eframe::egui::{self, Align, Layout, RichText};
use egui_phosphor::regular as icon;

use crate::files;
use crate::theme::{self, Palette};
use crate::widgets::{muted, primary_button, strong};

/// Qué se eligió en la pantalla de inicio.
pub enum Start {
    New,
    Open,
    Example,
    /// La experiencia de cálculo rápido, en su propia pantalla.
    Quick,
    Recent(PathBuf),
    /// Volver al proyecto abierto.
    Back,
}

/// Ancho de cada tarjeta de experiencia cuando caben las dos en un renglón.
const CARD_W: f32 = 360.0;

/// Si las dos tarjetas caben lado a lado en `width` px.
pub fn side_by_side(width: f32) -> bool {
    width >= 2.0 * CARD_W + theme::SP4 + 2.0 * theme::SP6
}

/// La pantalla de inicio. `open_name` es el proyecto abierto, al que se
/// puede volver.
pub fn show(ui: &mut egui::Ui, p: &Palette, recent: &[PathBuf], open_name: &str) -> Option<Start> {
    let mut chosen = None;
    egui::ScrollArea::vertical()
        .id_salt("inicio")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let width = ui.available_width();
            let two = side_by_side(width);
            let column = if two {
                2.0 * CARD_W + theme::SP4
            } else {
                (width - 2.0 * theme::SP4).max(0.0)
            };
            let left = ((width - column) / 2.0).max(0.0);
            ui.add_space(if two { theme::SP8 * 2.0 } else { theme::SP6 });
            ui.horizontal(|ui| {
                ui.add_space(left);
                ui.vertical(|ui| {
                    ui.set_width(column);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(icon::LIGHTNING)
                                .size(theme::VIEW_TITLE)
                                .color(p.accent),
                        );
                        ui.label(strong("IEEStudio").size(theme::VIEW_TITLE));
                    });
                    ui.label(muted(
                        p,
                        "Cálculo de instalaciones eléctricas de baja tensión · NOM-001-SEDE-2012",
                    ));
                    ui.add_space(theme::SP6);
                    let card_w = if two { CARD_W } else { column };
                    let mut picked = None;
                    let mut quick = false;
                    if two {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = theme::SP4;
                            // Cada tarjeta en su columna, de arriba abajo: así
                            // sus botones se acomodan a su ancho.
                            let column = Layout::top_down(Align::Min);
                            picked = ui
                                .allocate_ui_with_layout(egui::vec2(card_w, 0.0), column, |ui| {
                                    project_card(ui, p, card_w, recent)
                                })
                                .inner;
                            quick = ui
                                .allocate_ui_with_layout(egui::vec2(card_w, 0.0), column, |ui| {
                                    quick_card(ui, p, card_w)
                                })
                                .inner;
                        });
                    } else {
                        picked = project_card(ui, p, card_w, recent);
                        ui.add_space(theme::SP4);
                        quick = quick_card(ui, p, card_w);
                    }
                    if quick {
                        picked = Some(Start::Quick);
                    }
                    if picked.is_some() {
                        chosen = picked;
                    }
                    ui.add_space(theme::SP6);
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(format!("{}  Volver a «{open_name}»", icon::ARROW_LEFT))
                            .clicked()
                        {
                            chosen = Some(Start::Back);
                        }
                    });
                    ui.add_space(theme::SP3);
                    ui.label(
                        muted(
                            p,
                            "IEEStudio calcula y verifica; la aprobación es de quien firma.",
                        )
                        .size(theme::FOOT),
                    );
                    ui.add_space(theme::SP6);
                });
            });
        });
    chosen
}

/// Recuadro de una experiencia: título con su ícono, para qué sirve y lo que
/// se puede hacer.
fn experience(
    ui: &mut egui::Ui,
    p: &Palette,
    width: f32,
    glyph: &str,
    title: &str,
    what: &str,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(p.raise)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(theme::RADIUS_WINDOW)
        .inner_margin(egui::Margin::same(24))
        .show(ui, |ui| {
            ui.set_width((width - 2.0 * theme::SP6).max(0.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(glyph).size(theme::TITLE).color(p.accent_text));
                ui.label(strong(title).size(theme::TITLE));
            });
            ui.add_space(theme::SP1);
            ui.add(egui::Label::new(muted(p, what)).wrap());
            ui.add_space(theme::SP4);
            add(ui);
        });
}

fn project_card(ui: &mut egui::Ui, p: &Palette, width: f32, recent: &[PathBuf]) -> Option<Start> {
    let mut chosen = None;
    experience(
        ui,
        p,
        width,
        icon::FOLDER_OPEN,
        "Proyecto completo",
        "Tableros y circuitos, unifilar, revisión y memoria de cálculo de una instalación.",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                if primary_button(ui, p, &format!("{}  Nuevo proyecto", icon::FILE_PLUS))
                    .on_hover_text("Un tablero vacío para empezar (Ctrl+N)")
                    .clicked()
                {
                    chosen = Some(Start::New);
                }
                if ui
                    .button(format!("{}  Abrir…", icon::FOLDER_OPEN))
                    .on_hover_text("Un archivo .iees que ya tienes (Ctrl+O)")
                    .clicked()
                {
                    chosen = Some(Start::Open);
                }
                if ui
                    .button(format!("{}  Ejemplo", icon::HOUSE))
                    .on_hover_text("Una casa con seis circuitos para conocer el programa")
                    .clicked()
                {
                    chosen = Some(Start::Example);
                }
            });
            let existing: Vec<&PathBuf> = recent.iter().filter(|p| p.exists()).take(5).collect();
            if files::WEB || existing.is_empty() {
                return;
            }
            crate::widgets::section(ui, p, "Recientes");
            for path in existing {
                let name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                let folder = path
                    .parent()
                    .map_or_else(String::new, |d| d.display().to_string());
                let row = crate::widgets::property(ui, p, &name, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(
                            egui::Label::new(muted(p, folder).size(theme::SECONDARY)).truncate(),
                        );
                    });
                })
                .interact(egui::Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                if row.clicked() {
                    chosen = Some(Start::Recent(path.clone()));
                }
            }
            crate::widgets::end_group(ui);
        },
    );
    chosen
}

fn quick_card(ui: &mut egui::Ui, p: &Palette, width: f32) -> bool {
    let mut open = false;
    experience(
        ui,
        p,
        width,
        icon::CALCULATOR,
        "Cálculo rápido",
        "Un circuito suelto: corriente, protección, conductor y caída de tensión, con su procedimiento. Sin crear un proyecto.",
        |ui| {
            open = ui
                .button(format!("{}  Abrir cálculo rápido", icon::ARROW_RIGHT))
                .on_hover_text("Las herramientas de cálculo directo (F8)")
                .clicked();
        },
    );
    open
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_experiences_stack_on_narrow_windows() {
        assert!(side_by_side(1440.0));
        assert!(side_by_side(1024.0));
        assert!(!side_by_side(768.0));
        assert!(!side_by_side(600.0));
        assert!(!side_by_side(390.0));
    }
}
