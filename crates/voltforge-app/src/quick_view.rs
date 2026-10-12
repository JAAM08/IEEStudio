//! Cálculo rápido de un solo circuito: corriente, protección, conductor y
//! caída de tensión, con su procedimiento y alternativas cuando no cumple.

use eframe::egui::{self, Align, Layout, RichText};
use voltforge_core::Size;
use voltforge_project::quick::{quick_circuit, QuickInput, QuickResult};
use voltforge_project::{size_label, Status, Step};

use crate::app::{combo, number, CONDUITS, INSULATIONS, KINDS, MATERIALS, SYSTEMS};
use crate::theme::{self, Palette};
use voltforge_rules::Profile;

pub struct QuickView {
    input: QuickInput,
    result: Result<QuickResult, String>,
}

impl Default for QuickView {
    fn default() -> Self {
        let input = QuickInput::default();
        let result = quick_circuit(Profile::nom_001_sede_2012(), &input);
        QuickView { input, result }
    }
}

pub(crate) fn caption(p: &Palette, text: &str) -> RichText {
    RichText::new(text.to_uppercase()).small().color(p.ink2)
}

/// Tarjetas numeradas con el procedimiento paso a paso.
pub(crate) fn steps_ui(ui: &mut egui::Ui, p: &Palette, steps: &[Step]) {
    let width = ui.available_width().min(1000.0);
    for (i, step) in steps.iter().enumerate() {
        card(p).show(ui, |ui| {
            ui.set_width(width - 30.0);
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
                        ui.label(RichText::new(note).color(p.ink2));
                    }
                    if let Some(cite) = voltforge_rules::cite_all(&step.references) {
                        ui.label(
                            RichText::new(format!("Referencia: {cite}"))
                                .small()
                                .color(p.accent),
                        );
                    }
                });
            });
        });
        ui.add_space(6.0);
    }
}

fn card(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.panel)
        .stroke(egui::Stroke::new(1.0, p.line))
        .corner_radius(8)
        .inner_margin(14)
}

impl QuickView {
    /// Hoja del cálculo en PDF.
    pub fn pdf(&self) -> Result<Vec<u8>, String> {
        let result = self.result.as_ref().map_err(Clone::clone)?;
        voltforge_reports::quick_pdf(&self.input, result)
    }

    /// Veredicto del cálculo actual, si se pudo calcular.
    pub fn verdict(&self) -> Option<voltforge_project::verdict::Verdict> {
        self.result.as_ref().ok().map(|r| r.verdict())
    }

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
            self.result = quick_circuit(Profile::nom_001_sede_2012(), &self.input);
        }
    }

    fn inputs(&mut self, ui: &mut egui::Ui, p: &Palette) -> bool {
        let muted = |t: &str| RichText::new(t).color(p.ink2);
        let input = &mut self.input;
        let mut changed = false;
        ui.label(caption(p, "Cálculo rápido"));
        ui.label(
            RichText::new("Circuito")
                .family(theme::semibold())
                .size(18.0),
        );
        ui.label(muted(
            "Corriente, protección y caída de tensión de un solo circuito, sin proyecto.",
        ));
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

                ui.label(muted("Tipo de carga"));
                changed |= ui
                    .checkbox(&mut input.continuous, "Continua (3 h o más)")
                    .changed();
                ui.end_row();

                ui.label(muted("Longitud"));
                changed |= number(ui, &mut input.length_m, 0.5, 0.0, 100_000.0, " m");
                ui.end_row();

                ui.label(muted("Conductor"));
                changed |= combo(ui, "rapido-material", &mut input.material, &MATERIALS);
                ui.end_row();

                ui.label(muted("Aislamiento"));
                changed |= combo(
                    ui,
                    "rapido-aislamiento",
                    &mut input.insulation,
                    &INSULATIONS,
                );
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

                if input.kind != voltforge_core::CircuitKind::LineToNeutral {
                    ui.label(muted("Neutro"));
                    changed |= ui
                        .checkbox(&mut input.has_neutral, "Lleva neutro")
                        .changed();
                    ui.end_row();
                }

                ui.label(muted("Temp. ambiente"));
                changed |= number(ui, &mut input.ambient_c, 0.5, -10.0, 80.0, " °C");
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
        let limit = self.input.limit_percent;
        let (drop_color, drop_verdict) = match r.status {
            Status::Complies => (p.ok, format!("✔ Cumple el límite de {limit:.2} %")),
            _ => (p.bad, format!("✖ Excede el límite de {limit:.2} %")),
        };
        let size = size_label(&self.input.size);
        let big = |t: String| RichText::new(t).family(theme::semibold()).size(30.0);

        ui.columns(3, |cols| {
            cols[0].vertical(|ui| {
                card(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(118.0);
                    ui.label(caption(p, "Corriente"));
                    ui.label(big(format!("{:.2} A", r.current_a)));
                    match &r.protection {
                        Ok(pr) => {
                            ui.label(
                                RichText::new(format!("De diseño: {:.2} A", pr.design_a))
                                    .monospace(),
                            );
                            ui.label(muted(if self.input.continuous {
                                "Carga continua: 125 %".to_string()
                            } else {
                                "Carga no continua".to_string()
                            }));
                        }
                        Err(_) => {
                            ui.label(muted("Corriente nominal de la carga".to_string()));
                        }
                    }
                });
            });
            cols[1].vertical(|ui| {
                card(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(118.0);
                    ui.label(caption(p, "Protección"));
                    match &r.protection {
                        Ok(pr) => {
                            let ok = pr.status == Status::Complies;
                            let color = if ok { p.ok } else { p.bad };
                            ui.label(
                                big(format!("{}P-{:.0} A", pr.poles, pr.breaker_a)).color(color),
                            );
                            let verdict = if ok {
                                format!(
                                    "✔ {size} cumple: {:.0} A permitidos",
                                    pr.allowable_ampacity_a
                                )
                            } else {
                                match &pr.minimum_size {
                                    Some(min) => format!(
                                        "✖ {size} no cumple: {:.0} A permitidos; mínimo {}",
                                        pr.allowable_ampacity_a,
                                        size_label(min)
                                    ),
                                    None => format!(
                                        "✖ {size} no cumple: {:.0} A permitidos",
                                        pr.allowable_ampacity_a
                                    ),
                                }
                            };
                            ui.label(RichText::new(verdict).color(color));
                            if let Some(ground) = &pr.grounding_size {
                                ui.label(muted(format!("Puesta a tierra: {}", size_label(ground))));
                            }
                            if let Some(conduit) = &pr.conduit {
                                ui.label(muted(format!("Tubería: {conduit}")));
                            }
                        }
                        Err(message) => {
                            ui.label(big("—".to_string()).color(p.na));
                            ui.label(RichText::new("No evaluado").color(p.na));
                            ui.label(muted(format!("{message}.")));
                        }
                    }
                });
            });
            cols[2].vertical(|ui| {
                card(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(118.0);
                    ui.label(caption(p, "Caída de tensión"));
                    ui.label(big(format!("{:.2} %", r.drop.drop_percent)).color(drop_color));
                    ui.label(RichText::new(drop_verdict).color(drop_color));
                    ui.label(
                        RichText::new(format!(
                            "{:.2} V de {:.0} V",
                            r.drop.drop_v, r.drop.reference_voltage_v
                        ))
                        .monospace(),
                    );
                });
            });
        });

        for warning in &r.warnings {
            ui.add_space(6.0);
            ui.label(RichText::new(format!("Aviso · {warning}")).color(p.na));
        }

        ui.add_space(14.0);
        ui.label(caption(p, "Procedimiento"));
        ui.add_space(2.0);
        steps_ui(ui, p, &r.steps);
    }

    fn recommendations(&mut self, ui: &mut egui::Ui, p: &Palette) -> bool {
        let muted = |t: &str| RichText::new(t).color(p.ink2);
        let mut apply = None;
        let (title, intro) = match &self.result {
            Ok(r)
                if r.status == Status::Complies
                    && !matches!(&r.protection, Ok(pr) if pr.status == Status::Fails) =>
            {
                (
                    "Resultado",
                    "La selección actual cumple; no hace falta cambiarla.",
                )
            }
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
            "Este cálculo revisa corriente, interruptor, ampacidad con sus factores, \
             puesta a tierra, tubería y caída de tensión. Supone lugar seco, carga \
             lineal, tierra desnuda y todos los conductores en la misma tubería.",
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
