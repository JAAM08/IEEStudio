//! Símbolos de las cargas, con el dibujo de la simbología de los planos
//! eléctricos (NMX-J-136): luminario, contacto, motor, salidas especiales y
//! tablero. Se usan donde se elige o se muestra el tipo de un circuito, para
//! que la interfaz y el unifilar hablen el mismo idioma.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use ieestudio_project::LoadKind;

use crate::theme::semibold;

/// Lo que se puede conectar a un tablero: un circuito de cierto tipo de carga
/// o un subtablero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Load(LoadKind),
    Subpanel,
}

/// Dibuja el símbolo centrado en `rect`.
pub fn paint(painter: &egui::Painter, rect: Rect, symbol: Symbol, color: Color32) {
    let c = rect.center();
    let size = rect.width().min(rect.height());
    let r = size * 0.36;
    let width = (size / 16.0).clamp(1.0, 2.2);
    let stroke = Stroke::new(width, color);
    let letters = |text: &str, scale: f32| {
        painter.text(
            c,
            egui::Align2::CENTER_CENTER,
            text,
            FontId::new(size * scale, semibold()),
            color,
        );
    };
    match symbol {
        // Luminario: dos circunferencias y la cruz.
        Symbol::Load(LoadKind::Lighting) => {
            painter.circle_stroke(c, r * 0.8, stroke);
            painter.circle_stroke(c, r * 0.38, stroke);
            painter.line_segment([c - Vec2::X * r * 1.3, c + Vec2::X * r * 1.3], stroke);
            painter.line_segment([c - Vec2::Y * r * 1.3, c + Vec2::Y * r * 1.3], stroke);
        }
        // Contacto: circunferencia con una diagonal.
        Symbol::Load(LoadKind::Receptacles) => {
            painter.circle_stroke(c, r, stroke);
            let d = r * 0.62;
            painter.line_segment([c + Vec2::new(-d, d), c + Vec2::new(d, -d)], stroke);
        }
        Symbol::Load(LoadKind::Motor) => {
            painter.circle_stroke(c, r, stroke);
            letters("M", 0.42);
        }
        // Salidas especiales: la circunferencia con sus letras.
        Symbol::Load(LoadKind::AirConditioning) => {
            painter.circle_stroke(c, r, stroke);
            letters("AA", 0.27);
        }
        Symbol::Load(LoadKind::Appliance) => {
            painter.circle_stroke(c, r, stroke);
            letters("AF", 0.27);
        }
        Symbol::Load(LoadKind::KitchenEquipment) => {
            painter.circle_stroke(c, r, stroke);
            letters("EC", 0.27);
        }
        // Otra carga: la flecha del unifilar.
        Symbol::Load(LoadKind::General) => {
            let top = c - Vec2::Y * r * 1.15;
            let base = c + Vec2::Y * r * 0.15;
            painter.line_segment([top, base], stroke);
            painter.add(egui::Shape::closed_line(
                vec![
                    base + Vec2::new(-r * 0.55, 0.0),
                    base + Vec2::new(r * 0.55, 0.0),
                    Pos2::new(c.x, c.y + r * 1.0),
                ],
                stroke,
            ));
        }
        // Tablero: rectángulo con una diagonal.
        Symbol::Subpanel => {
            let half = Vec2::new(r * 1.25, r * 0.65);
            let body = Rect::from_center_size(c, half * 2.0);
            painter.rect_stroke(body, 0.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment([body.left_bottom(), body.right_top()], stroke);
        }
    }
}

/// El símbolo como un elemento más de la línea, de `size` puntos.
pub fn show(ui: &mut egui::Ui, symbol: Symbol, size: f32, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint(ui.painter(), rect, symbol, color);
    response
}
