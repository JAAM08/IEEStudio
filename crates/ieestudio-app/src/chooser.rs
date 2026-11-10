//! «¿Qué vas a conectar?»: al agregar un circuito se elige primero el tipo de
//! carga, para que el programa pida los datos de ese equipo y le aplique sus
//! reglas. Así un motor o un aire acondicionado no se capturan como carga
//! general por no saber dónde estaban.

use eframe::egui::{self, Align, Key, Layout, RichText, Sense, Vec2};
use ieestudio_project::LoadKind;
use ieestudio_rules::{Profile, Rule};

use crate::state::Doc;
use crate::symbols::{self, Symbol};
use crate::theme::{semibold, Palette};
use crate::widgets::{muted, strong};

/// Lo que pasó con la ventana en este cuadro.
pub enum Outcome {
    Open,
    Cancel,
    Pick(Symbol),
}

struct Option_ {
    symbol: Symbol,
    title: &'static str,
    sub: &'static str,
    rules: &'static [Rule],
}

const OPTIONS: [Option_; 8] = [
    Option_ {
        symbol: Symbol::Load(LoadKind::Lighting),
        title: "Alumbrado",
        sub: "Luminarios del alumbrado general.",
        rules: &[Rule::LightingLoad, Rule::LightingDemand],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::Receptacles),
        title: "Contactos",
        sub: "Contactos de uso general, por número de salidas.",
        rules: &[Rule::ReceptacleOutletLoad, Rule::ReceptacleDemand],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::Motor),
        title: "Motor",
        sub: "Bombas, ventiladores, extractores, compresores de aire. Se elige por hp.",
        rules: &[
            Rule::MotorFullLoadCurrent,
            Rule::MotorBranchConductor,
            Rule::MotorBranchProtection,
        ],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::AirConditioning),
        title: "Aire acondicionado",
        sub: "Minisplit, equipo paquete o refrigeración. Con los datos de su placa.",
        rules: &[
            Rule::CompressorRating,
            Rule::CompressorConductor,
            Rule::EquipmentNameplate,
        ],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::Appliance),
        title: "Aparato fijo",
        sub: "Calentador de agua, lavavajillas, triturador…",
        rules: &[Rule::ApplianceDemand],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::KitchenEquipment),
        title: "Equipo de cocina",
        sub: "Cocina comercial: hornos, freidoras, planchas.",
        rules: &[Rule::KitchenDemand],
    },
    Option_ {
        symbol: Symbol::Load(LoadKind::General),
        title: "Otra carga",
        sub: "Cualquier otra carga, en watts. Va al 100 % en el alimentador.",
        rules: &[Rule::DesignCurrent],
    },
    Option_ {
        symbol: Symbol::Subpanel,
        title: "Subtablero",
        sub: "Un tablero que se alimenta de este. Es el mismo que se agrega en Tableros.",
        rules: &[Rule::FeederLoad],
    },
];

const KEYS: [Key; 8] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
];

const TILE: Vec2 = Vec2::new(196.0, 156.0);
const GAP: f32 = 12.0;

/// Ventana para elegir qué se conecta al tablero `panel`.
pub fn show(ctx: &egui::Context, p: &Palette, doc: &Doc, panel: usize) -> Outcome {
    let profile = Profile::by_id(&doc.project.profile).unwrap_or(Profile::nom_001_sede_2012());
    let panel_id = doc
        .project
        .panels
        .get(panel)
        .map_or(String::new(), |x| x.id.clone());
    let mut picked = None;
    for (k, key) in KEYS.iter().enumerate() {
        if ctx.input(|i| i.key_pressed(*key) && i.modifiers.is_none()) {
            picked = Some(OPTIONS[k].symbol);
        }
    }
    let modal = egui::Modal::new(egui::Id::new("que-vas-a-conectar"))
        .frame(
            egui::Frame::new()
                .fill(p.panel)
                .stroke(egui::Stroke::new(1.0, p.line))
                .corner_radius(10)
                .inner_margin(egui::Margin::symmetric(24, 20)),
        )
        .show(ctx, |ui| {
            ui.set_width(4.0 * TILE.x + 3.0 * GAP);
            ui.label(strong(format!("¿Qué vas a conectar a {panel_id}?")).size(20.0));
            ui.add_space(2.0);
            ui.label(muted(
                p,
                "Según lo que elijas, IEEStudio te pide solo los datos de ese equipo y le aplica sus reglas.",
            ));
            ui.add_space(14.0);
            for row in OPTIONS.chunks(4).enumerate() {
                let (r, options) = row;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for (k, option) in options.iter().enumerate() {
                        if tile(ui, p, profile, option, r * 4 + k + 1) {
                            picked = Some(option.symbol);
                        }
                    }
                });
                ui.add_space(GAP);
            }
            ui.horizontal(|ui| {
                ui.label(muted(p, "Los símbolos son los mismos del unifilar. También puedes elegir con las teclas 1 a 8.").size(12.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Cancelar").clicked() {
                        ui.close();
                    }
                });
            });
        });
    match picked {
        Some(symbol) => Outcome::Pick(symbol),
        None if modal.should_close() => Outcome::Cancel,
        None => Outcome::Open,
    }
}

/// Una tarjeta: símbolo, número, nombre, para qué es y sus artículos.
fn tile(
    ui: &mut egui::Ui,
    p: &Palette,
    profile: &Profile,
    option: &Option_,
    number: usize,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(TILE, Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter().clone();
    painter.rect(
        rect,
        8.0,
        p.raise,
        egui::Stroke::new(
            if hovered { 1.5 } else { 1.0 },
            if hovered { p.accent } else { p.line },
        ),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink2(Vec2::new(14.0, 12.0));
    let icon = egui::Rect::from_min_size(inner.min, Vec2::splat(38.0));
    symbols::paint(
        &painter,
        icon,
        option.symbol,
        if hovered { p.accent_text } else { p.ink },
    );
    painter.text(
        inner.right_top(),
        egui::Align2::RIGHT_TOP,
        number.to_string(),
        egui::FontId::monospace(11.0),
        p.ink3,
    );
    let clauses: Vec<String> = option.rules.iter().map(|r| profile.clause(*r)).collect();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_max(
                inner.min + Vec2::new(0.0, 46.0),
                inner.max,
            ))
            .layout(Layout::top_down(Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 3.0;
    child.label(RichText::new(option.title).size(14.5).family(semibold()));
    child.add(egui::Label::new(muted(p, option.sub).size(12.0)).wrap());
    painter.text(
        inner.left_bottom(),
        egui::Align2::LEFT_BOTTOM,
        first_clause(&clauses),
        egui::FontId::monospace(10.5),
        p.ink3,
    );
    let tip: Vec<String> = option
        .rules
        .iter()
        .zip(&clauses)
        .map(|(rule, clause)| match profile.rule_description(*rule) {
            Some(text) => format!("{clause}: {text}"),
            None => clause.clone(),
        })
        .collect();
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip.join("\n"))
        .clicked()
}

/// La primera cláusula, corta para que quepa en la tarjeta, y cuántas más hay.
fn first_clause(clauses: &[String]) -> String {
    let Some(first) = clauses.first() else {
        return String::new();
    };
    let short: String = if first.chars().count() > 24 {
        let cut: String = first.chars().take(23).collect();
        format!("{}…", cut.trim_end_matches([',', ' ']))
    } else {
        first.clone()
    };
    match clauses.len() {
        0 | 1 => short,
        n => format!("{short} +{}", n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_of_load_can_be_chosen() {
        for kind in [
            LoadKind::General,
            LoadKind::Lighting,
            LoadKind::Receptacles,
            LoadKind::Appliance,
            LoadKind::KitchenEquipment,
            LoadKind::Motor,
            LoadKind::AirConditioning,
        ] {
            assert!(
                OPTIONS.iter().any(|o| o.symbol == Symbol::Load(kind)),
                "falta {kind:?}"
            );
        }
        assert!(OPTIONS.iter().any(|o| o.symbol == Symbol::Subpanel));
    }

    #[test]
    fn every_rule_shown_has_its_clause() {
        let profile = Profile::nom_001_sede_2012();
        for option in &OPTIONS {
            for rule in option.rules {
                assert!(
                    profile.rule_description(*rule).is_some(),
                    "{} sin referencia",
                    rule.id()
                );
            }
        }
    }

    #[test]
    fn long_clauses_are_shortened() {
        let clauses = vec![
            "430-6(a)(1), Tablas 430-248 y 430-250".to_string(),
            "430-22".to_string(),
        ];
        let text = first_clause(&clauses);
        assert!(text.ends_with("+1"));
        assert!(text.chars().count() <= 28);
    }
}
