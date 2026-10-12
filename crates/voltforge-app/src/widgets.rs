//! Piezas de interfaz que comparten todos los paneles.

use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke};
use egui_phosphor::regular as icon;
use voltforge_core::{CircuitKind, ConduitType, Insulation, Material, System};
use voltforge_project::verdict::{Calculation, Compliance, Verdict};
use voltforge_project::{LoadKind, Status, Step};

use crate::theme::{semibold, Palette};

pub const SYSTEMS: [(&str, System); 6] = [
    ("1F-2H 127 V", System::SINGLE_127),
    ("1F-3H 240/120 V", System::SPLIT_240_120),
    ("3F-4H 220Y/127 V", System::WYE_220_127),
    ("3F-4H 208Y/120 V", System::WYE_208_120),
    ("3F-4H 480Y/277 V", System::WYE_480_277),
    ("3F-3H 440 V", System::DELTA_440),
];

pub const KINDS: [(&str, CircuitKind); 3] = [
    ("1 fase", CircuitKind::LineToNeutral),
    ("2 fases", CircuitKind::LineToLine),
    ("3 fases", CircuitKind::ThreePhase),
];

pub const MATERIALS: [(&str, Material); 2] = [
    ("Cobre", Material::Copper),
    ("Aluminio", Material::Aluminum),
];

pub const INSULATIONS: [(&str, Insulation); 12] = [
    ("TW", Insulation::Tw),
    ("THW", Insulation::Thw),
    ("THHW", Insulation::Thhw),
    ("THW-2", Insulation::Thw2),
    ("THHN", Insulation::Thhn),
    ("THWN", Insulation::Thwn),
    ("THWN-2", Insulation::Thwn2),
    ("XHHW", Insulation::Xhhw),
    ("XHHW-2", Insulation::Xhhw2),
    ("RHH", Insulation::Rhh),
    ("RHW", Insulation::Rhw),
    ("RHW-2", Insulation::Rhw2),
];

pub const CONDUITS: [(&str, ConduitType); 5] = [
    ("EMT", ConduitType::Emt),
    ("IMC", ConduitType::Imc),
    ("RMC", ConduitType::Rmc),
    ("PVC cédula 40", ConduitType::Pvc40),
    ("PVC cédula 80", ConduitType::Pvc80),
];

pub const LOAD_KINDS: [(&str, LoadKind); 4] = [
    ("General", LoadKind::General),
    ("Alumbrado", LoadKind::Lighting),
    ("Motor", LoadKind::Motor),
    ("Aire acondicionado", LoadKind::AirConditioning),
];

pub fn label_of<T: PartialEq + Copy>(options: &[(&'static str, T)], value: T) -> &'static str {
    options
        .iter()
        .find(|(_, v)| *v == value)
        .map_or("?", |(l, _)| l)
}

/// Lista desplegable sobre un conjunto fijo de opciones. Devuelve `true` si cambió.
pub fn combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut T,
    options: &[(&'static str, T)],
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(label_of(options, *value))
        .show_ui(ui, |ui| {
            for (label, option) in options {
                changed |= ui.selectable_value(value, *option, *label).changed();
            }
        });
    changed
}

/// Lista desplegable con «del proyecto» como primera opción.
pub fn combo_override<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut Option<T>,
    options: &[(&'static str, T)],
    inherited: &str,
) -> bool {
    let mut changed = false;
    let text = match value {
        Some(v) => label_of(options, *v).to_string(),
        None => format!("{inherited} · proyecto"),
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(text)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, None, format!("{inherited} · proyecto"))
                .changed();
            for (label, option) in options {
                changed |= ui.selectable_value(value, Some(*option), *label).changed();
            }
        });
    changed
}

pub fn number(
    ui: &mut egui::Ui,
    value: &mut f64,
    speed: f64,
    min: f64,
    max: f64,
    suffix: &str,
) -> bool {
    ui.add(
        egui::DragValue::new(value)
            .speed(speed)
            .range(min..=max)
            .suffix(suffix),
    )
    .changed()
}

pub fn status_color(p: &Palette, status: Status) -> Color32 {
    match status {
        Status::Complies => p.ok,
        Status::Fails => p.bad,
        Status::NotEvaluated => p.warn,
    }
}

pub fn status_icon(status: Status) -> &'static str {
    match status {
        Status::Complies => icon::CHECK_CIRCLE,
        Status::Fails => icon::X_CIRCLE,
        Status::NotEvaluated => icon::CIRCLE_DASHED,
    }
}

/// Estado con ícono y texto: nunca solo color.
pub fn status_text(p: &Palette, status: Status) -> RichText {
    let label = match status {
        Status::Complies => "Cumple",
        Status::Fails => "No cumple",
        Status::NotEvaluated => "Sin evaluar",
    };
    RichText::new(format!("{} {label}", status_icon(status))).color(status_color(p, status))
}

/// Solo el ícono de estado, para columnas angostas y árboles.
pub fn status_glyph(p: &Palette, status: Status) -> RichText {
    RichText::new(status_icon(status)).color(status_color(p, status))
}

/// El peor estado de una lista: no cumple > sin evaluar > cumple.
pub fn worst(statuses: impl IntoIterator<Item = Status>) -> Status {
    statuses
        .into_iter()
        .fold(Status::Complies, |acc, s| match (acc, s) {
            (Status::Fails, _) | (_, Status::Fails) => Status::Fails,
            (Status::NotEvaluated, _) | (_, Status::NotEvaluated) => Status::NotEvaluated,
            _ => Status::Complies,
        })
}

pub fn muted(p: &Palette, text: impl Into<String>) -> RichText {
    RichText::new(text).color(p.ink2)
}

pub fn strong(text: impl Into<String>) -> RichText {
    RichText::new(text).family(semibold())
}

/// Encabezado de sección dentro de un panel.
pub fn section(ui: &mut egui::Ui, p: &Palette, title: &str) {
    ui.add_space(8.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .size(10.5)
            .color(p.ink3)
            .family(semibold()),
    );
    ui.add_space(1.0);
}

/// Procedencia de un valor en el panel de propiedades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Lo calculó VoltForge.
    Calculated,
    /// VoltForge lo propone; el usuario puede fijar otro.
    Proposed,
    /// Lo fijó el usuario; VoltForge lo verifica y no lo cambia.
    Fixed,
}

pub fn source_badge(ui: &mut egui::Ui, p: &Palette, source: Source) {
    let (text, color, tip) = match source {
        Source::Calculated => (
            "calculado",
            p.ink3,
            "Resultado de VoltForge con los datos y las tablas del perfil.",
        ),
        Source::Proposed => (
            "propuesto",
            p.accent_text,
            "El menor valor que cumple. Es una propuesta: puedes fijar otro y se verificará.",
        ),
        Source::Fixed => (
            "fijado por ti",
            p.ink2,
            "Lo decidiste tú. VoltForge lo verifica, nunca lo cambia.",
        ),
    };
    egui::Frame::new()
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.6)))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(5, 0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.5).color(color));
        })
        .response
        .on_hover_text(tip);
}

/// Fila de propiedad: etiqueta a la izquierda, valor o control a la derecha.
pub fn property(ui: &mut egui::Ui, p: &Palette, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        fixed_label(ui, p, label);
        add(ui);
    });
}

/// Etiqueta de ancho fijo, alineada a la izquierda, para formar columnas.
pub fn fixed_label(ui: &mut egui::Ui, p: &Palette, label: &str) {
    ui.allocate_ui_with_layout(
        egui::vec2(124.0, 20.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(124.0);
            ui.add(egui::Label::new(muted(p, label)).truncate());
        },
    );
}

/// Grupo del procedimiento al que pertenece un paso.
fn step_group(title: &str) -> &'static str {
    match title {
        "Corriente nominal" | "Corriente del circuito" | "Corriente de diseño" => "Corriente",
        "Interruptor" => "Protección",
        "Ampacidad del conductor" | "Verificación del conductor" | "Puesta a tierra de equipos" => {
            "Conductor y puesta a tierra"
        }
        "Tubería conduit" => "Tubería",
        _ => "Caída de tensión",
    }
}

/// Procedimiento agrupado en secciones contraíbles. Cada paso muestra la
/// fórmula, los valores sustituidos, la nota y su referencia normativa.
pub fn steps_ui(
    ui: &mut egui::Ui,
    p: &Palette,
    id: impl std::hash::Hash + std::fmt::Debug + Copy,
    steps: &[Step],
    open: Option<bool>,
) {
    let mut groups: Vec<(&str, Vec<(usize, &Step)>)> = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let g = step_group(&step.title);
        match groups.iter_mut().find(|(name, _)| *name == g) {
            Some((_, list)) => list.push((i, step)),
            None => groups.push((g, vec![(i, step)])),
        }
    }
    for (name, list) in groups {
        let header = egui::CollapsingHeader::new(strong(name))
            .id_salt((id, name))
            .default_open(true)
            .open(open);
        header.show(ui, |ui| {
            for (i, step) in list {
                ui.horizontal_top(|ui| {
                    ui.add_sized(
                        [22.0, 18.0],
                        egui::Label::new(
                            RichText::new(format!("{}", i + 1))
                                .family(semibold())
                                .color(p.accent_text),
                        ),
                    );
                    ui.vertical(|ui| {
                        ui.label(strong(&step.title));
                        if let Some(formula) = &step.formula {
                            ui.label(RichText::new(formula).monospace().color(p.ink2));
                        }
                        ui.label(RichText::new(&step.substitution).monospace());
                        if let Some(note) = &step.note {
                            ui.label(muted(p, note));
                        }
                        if let Some(cite) = voltforge_rules::cite_all(&step.references) {
                            ui.label(
                                RichText::new(format!("{} {cite}", icon::BOOK_OPEN))
                                    .size(11.0)
                                    .color(p.ink3),
                            );
                        }
                    });
                });
                ui.add_space(6.0);
            }
        });
    }
}

/// Mensaje centrado para un panel sin contenido.
pub fn empty_state(ui: &mut egui::Ui, p: &Palette, glyph: &str, title: &str, hint: &str) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(ui.available_height() * 0.3);
        ui.label(RichText::new(glyph).size(28.0).color(p.ink3));
        ui.label(strong(title).color(p.ink2));
        ui.label(muted(p, hint));
    });
}

/// Los tres niveles del veredicto, con su explicación al pasar el cursor.
pub fn verdict_chips(ui: &mut egui::Ui, p: &Palette, verdict: &Verdict) {
    let lines = verdict.lines("NOM-001-SEDE-2012");
    let short = [
        match verdict.calculation {
            Calculation::Complete => "Cálculo completo",
            Calculation::Incomplete => "Cálculo incompleto",
        },
        match verdict.compliance {
            Compliance::Conforms => "Conforme",
            Compliance::DoesNotConform => "No conforme",
            Compliance::Pending => "Verificación pendiente",
        },
        "Aprobación profesional pendiente",
    ];
    for (i, (level, status, detail)) in lines.iter().enumerate().rev() {
        let color = if i == 2 {
            p.ink2
        } else {
            status_color(p, *status)
        };
        let glyph = match (i, status) {
            (2, _) => icon::SEAL_QUESTION,
            (_, Status::Complies) => icon::CHECK_CIRCLE,
            (_, Status::Fails) => icon::X_CIRCLE,
            (_, Status::NotEvaluated) => icon::CIRCLE_DASHED,
        };
        ui.label(RichText::new(format!("{glyph} {}", short[i])).color(color))
            .on_hover_text(format!("{level}: {detail}"));
        if i > 0 {
            ui.label(muted(p, "·"));
        }
    }
}
