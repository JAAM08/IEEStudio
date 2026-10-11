//! Cálculo rápido: caída de tensión de un solo circuito, con su
//! procedimiento y alternativas cuando no cumple.

use eframe::egui::{self, Align, Layout, RichText};
use voltforge_core::{Profile, Size};
use voltforge_project::quick::{quick_voltage_drop, size_label, QuickInput, QuickResult};
use voltforge_project::Status;

use crate::app::{combo, number, CONDUITS, KINDS, MATERIALS, SYSTEMS};
use crate::theme::{self, Palette};

pub struct QuickView {
    input: QuickInput,
    result: Result<QuickResult, String>,
}

impl Default for QuickView {
    fn default() -> Self {
        let input = QuickInput::default();
        let result = quick_voltage_drop(Profile::nom_001_sede_2012(), &input);
        QuickView { input, result }
    }
}

fn caption(p: &Palette, text: &str) -> RichText {
    RichText::new(text.to_uppercase()).small().color(p.ink2)
}

fn card(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(8)
        .inner_margin(14)
}

impl QuickView {
    /// Texto de la barra inferior.
    pub fn summary(&self) -> String {
        let adjust = match self.input.resistance_temperature_c {
            Some(t) => format!("a {t:.0} °C"),
            None => "apagado".to_string(),
        };
        format!("Perfil NOM-001-SEDE-2012 · Ajuste de resistencia por temperatura: {adjust}")
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let side = egui::Frame::new().fill(p.panel).inner_margin(14);
        let mut changed = false;
        egui::Panel::left("rapido-datos")
            .frame(side)
            .default_size(300.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| changed |= self.inputs(ui, p));
            });
        egui::Panel::right("rapido-recomendaciones")
            .frame(side)
            .default_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| changed |= self.recommendations(ui, p));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg).inner_margin(16))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| self.procedure(ui, p));
            });
        if changed {
            self.result = quick_voltage_drop(Profile::nom_001_sede_2012(), &self.input);
        }
    }

    fn inputs(&mut self, ui: &mut egui::Ui, p: &Palette) -> bool {
        let muted = |t: &str| RichText::new(t).color(p.ink2);
        let input = &mut self.input;
        let mut changed = false;
        ui.label(caption(p, "Cálculo rápido"));
        ui.label(
            RichText::new("Caída de tensión")
                .family(theme::semibold())
                .size(18.0),
        );
        ui.label(muted("Un solo circuito, sin cuadro de cargas."));
        ui.add_space(10.0);

        egui::Grid::new("rapido-grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label(muted("Sistema"));
                changed |= combo(ui, "rapido-sistema", &mut input.system, &SYSTEMS);
                ui.end_row();

                ui.label(muted("Conexión"));
                changed |= combo(ui, "rapido-conexion", &mut input.kind, &KINDS);
                ui.end_row();

                ui.label(muted("Carga en"));
                ui.horizontal(|ui| {
                    changed |= ui
                        .radio_value(&mut input.load_in_amperes, false, "watts")
                        .changed();
                    changed |= ui
                        .radio_value(&mut input.load_in_amperes, true, "amperes")
                        .changed();
                });
                ui.end_row();

                if input.load_in_amperes {
                    ui.label(muted("Corriente"));
                    changed |= number(ui, &mut input.current_a, 0.5, 0.0, 10_000.0, " A");
                } else {
                    ui.label(muted("Carga"));
                    changed |= number(ui, &mut input.power_w, 50.0, 0.0, 10_000_000.0, " W");
                }
                ui.end_row();

                ui.label(muted("Factor de potencia"));
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut input.power_factor)
                            .speed(0.005)
                            .range(0.1..=1.0)
                            .fixed_decimals(2),
                    )
                    .changed();
                ui.end_row();

                ui.label(muted("Longitud"));
                changed |= number(ui, &mut input.length_m, 0.5, 0.0, 100_000.0, " m");
                ui.end_row();

                ui.label(muted("Conductor"));
                changed |= combo(ui, "rapido-material", &mut input.material, &MATERIALS);
                ui.end_row();

                ui.label(muted("Calibre"));
                let profile = Profile::nom_001_sede_2012();
                let raceway = input.conduit.raceway_material();
                egui::ComboBox::from_id_salt("rapido-calibre")
                    .selected_text(size_label(&input.size))
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for c in profile.sizes.iter().skip_while(|c| c.size != "14") {
                            let size = Size::new(&c.size);
                            // Solo los calibres que la Tabla 9 cubre.
                            if profile.impedance(&size, input.material, raceway).is_err() {
                                continue;
                            }
                            let label = size_label(&size);
                            changed |= ui.selectable_value(&mut input.size, size, label).changed();
                        }
                    });
                ui.end_row();

                ui.label(muted("Conductores por fase"));
                changed |= ui
                    .add(egui::DragValue::new(&mut input.conductors_per_phase).range(1..=12))
                    .changed();
                ui.end_row();

                ui.label(muted("Canalización"));
                changed |= combo(ui, "rapido-tubo", &mut input.conduit, &CONDUITS);
                ui.end_row();

                ui.label(muted("Límite de caída"));
                changed |= number(ui, &mut input.limit_percent, 0.05, 0.1, 20.0, " %");
                ui.end_row();
            });

        ui.add_space(8.0);
        let mut adjust = input.resistance_temperature_c.is_some();
        if ui
            .checkbox(&mut adjust, "Ajustar resistencia por temperatura")
            .changed()
        {
            input.resistance_temperature_c = adjust.then_some(60.0);
            changed = true;
        }
        if let Some(t) = &mut input.resistance_temperature_c {
            ui.horizontal(|ui| {
                ui.label(muted("Temperatura del conductor"));
                changed |= number(ui, t, 0.5, 0.0, 90.0, " °C");
            });
        }
        changed
    }

    fn procedure(&self, ui: &mut egui::Ui, p: &Palette) {
        let muted = |t: String| RichText::new(t).color(p.ink2);
        let r = match &self.result {
            Ok(r) => r,
            Err(message) => {
                card(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("— No evaluado")
                            .color(p.na)
                            .family(theme::semibold())
                            .size(18.0),
                    );
                    ui.label(muted(format!("No se pudo calcular: {message}.")));
                });
                return;
            }
        };
        let (color, verdict) = match r.status {
            Status::Complies => (
                p.ok,
                format!(
                    "✔ Cumple: no excede el límite de {:.2} %",
                    self.input.limit_percent
                ),
            ),
            _ => (
                p.bad,
                format!(
                    "✖ No cumple: excede el límite de {:.2} %",
                    self.input.limit_percent
                ),
            ),
        };

        card(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caption(p, "Caída de tensión"));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{:.2} %", r.drop.drop_percent))
                        .family(theme::semibold())
                        .size(40.0)
                        .color(color),
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(format!(
                            "{:.2} V de {:.0} V",
                            r.drop.drop_v, r.drop.reference_voltage_v
                        ))
                        .monospace(),
                    );
                    ui.label(muted(format!("Corriente: {:.2} A", r.current_a)));
                });
            });
            ui.label(RichText::new(verdict).color(color));
        });

        for warning in &r.warnings {
            ui.add_space(6.0);
            ui.label(RichText::new(format!("Aviso · {warning}")).color(p.na));
        }

        ui.add_space(14.0);
        ui.label(caption(p, "Procedimiento"));
        ui.add_space(2.0);
        for (i, step) in r.steps.iter().enumerate() {
            card(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    ui.label(
                        RichText::new(format!("{}", i + 1))
                            .family(theme::semibold())
                            .size(16.0)
                            .color(p.accent),
                    );
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&step.title).family(theme::semibold()));
                        if let Some(formula) = &step.formula {
                            ui.label(RichText::new(formula).monospace().color(p.ink2));
                        }
                        ui.label(RichText::new(&step.substitution).monospace());
                        if let Some(note) = &step.note {
                            ui.label(muted(note.clone()));
                        }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    fn recommendations(&mut self, ui: &mut egui::Ui, p: &Palette) -> bool {
        let muted = |t: &str| RichText::new(t).color(p.ink2);
        let mut apply = None;
        let (title, intro) = match &self.result {
            Ok(r) if r.status == Status::Complies => (
                "Resultado",
                "La selección actual cumple; no hace falta cambiarla.",
            ),
            _ => (
                "Recomendaciones",
                "Alternativas ya calculadas. Ninguna se aplica sin que la elijas.",
            ),
        };
        ui.label(caption(p, title));
        ui.label(muted(intro));
        ui.add_space(8.0);

        if let Ok(r) = &self.result {
            for rec in &r.recommendations {
                egui::Frame::new()
                    .fill(p.raise)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(8)
                    .inner_margin(12)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&rec.title).family(theme::semibold()));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(&rec.value).monospace().color(p.ok));
                            });
                        });
                        ui.label(muted(&rec.detail));
                        if let Some(choice) = &rec.apply {
                            ui.add_space(4.0);
                            if ui.button("Aplicar").clicked() {
                                apply = Some(choice.clone());
                            }
                        }
                    });
                ui.add_space(8.0);
            }
        }

        ui.add_space(6.0);
        ui.label(muted(
            "Este cálculo revisa la caída de tensión y compara la corriente con la \
             ampacidad de tabla, sin factores de corrección. La temperatura ambiente, \
             el agrupamiento y la protección se verifican en el proyecto.",
        ));

        match apply {
            Some((size, count)) => {
                self.input.size = size;
                self.input.conductors_per_phase = count;
                true
            }
            None => false,
        }
    }
}
