//! Pantalla principal: árbol del proyecto, cuadro de cargas y detalle.

use std::path::PathBuf;

use eframe::egui::{self, Align, Color32, Layout, RichText, Sense};
use egui_extras::{Column, TableBuilder};
use voltforge_core::sizing::Insulation;
use voltforge_core::{CircuitKind, ConduitType, Material, Profile, Size, System};
use voltforge_project::{
    calculate, example, Circuit, Feeder, Load, LoadKind, Overrides, Panel, PanelResult, Phase,
    Project, ProjectResult, RunResult, Settings, Source, Status, FORMAT_VERSION,
};

use crate::quick_view::{caption, steps_ui, QuickView};
use crate::theme::{self, Palette, ACCENTS};

pub(crate) const SYSTEMS: [(&str, System); 6] = [
    ("1F-2H 127 V", System::SINGLE_127),
    ("1F-3H 240/120 V", System::SPLIT_240_120),
    ("3F-4H 220Y/127 V", System::WYE_220_127),
    ("3F-4H 208Y/120 V", System::WYE_208_120),
    ("3F-4H 480Y/277 V", System::WYE_480_277),
    ("3F-3H 440 V", System::DELTA_440),
];

pub(crate) const KINDS: [(&str, CircuitKind); 3] = [
    ("1 fase", CircuitKind::LineToNeutral),
    ("2 fases", CircuitKind::LineToLine),
    ("3 fases", CircuitKind::ThreePhase),
];

pub(crate) const MATERIALS: [(&str, Material); 2] = [
    ("Cobre", Material::Copper),
    ("Aluminio", Material::Aluminum),
];

pub(crate) const INSULATIONS: [(&str, Insulation); 12] = [
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

pub(crate) const CONDUITS: [(&str, ConduitType); 5] = [
    ("EMT", ConduitType::Emt),
    ("IMC", ConduitType::Imc),
    ("RMC", ConduitType::Rmc),
    ("PVC cédula 40", ConduitType::Pvc40),
    ("PVC cédula 80", ConduitType::Pvc80),
];

const LOAD_KINDS: [(&str, LoadKind); 4] = [
    ("General", LoadKind::General),
    ("Alumbrado", LoadKind::Lighting),
    ("Motor", LoadKind::Motor),
    ("Aire acondicionado", LoadKind::AirConditioning),
];

fn label_of<T: PartialEq + Copy>(options: &[(&'static str, T)], value: T) -> &'static str {
    options
        .iter()
        .find(|(_, v)| *v == value)
        .map_or("?", |(l, _)| l)
}

/// Lista desplegable sobre un conjunto fijo de opciones. Devuelve `true` si cambió.
pub(crate) fn combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut T,
    options: &[(&'static str, T)],
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(label_of(options, *value))
        .width(150.0)
        .show_ui(ui, |ui| {
            for (label, option) in options {
                changed |= ui.selectable_value(value, *option, *label).changed();
            }
        });
    changed
}

pub(crate) fn number(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    Feeder,
    Circuit(usize),
}

/// Captura de pantalla pedida por línea de comandos, para pruebas visuales.
struct Capture {
    path: PathBuf,
    frames: u32,
}

/// Vista que ocupa la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Project,
    Quick,
}

pub struct App {
    view: View,
    quick: QuickView,
    project: Project,
    path: Option<PathBuf>,
    dirty: bool,
    result: Result<ProjectResult, String>,
    panel: usize,
    selection: Selection,
    dark: bool,
    accent: usize,
    palette: Palette,
    notice: Option<String>,
    /// Aviso de una acción terminada, por ejemplo un PDF guardado.
    done: Option<String>,
    capture: Option<Capture>,
}

fn empty_project() -> Project {
    Project {
        format_version: FORMAT_VERSION,
        name: "Proyecto nuevo".to_string(),
        profile: "nom-001-sede-2012".to_string(),
        settings: Settings::default(),
        source: Source {
            name: "Acometida".to_string(),
            system: System::WYE_220_127,
        },
        panels: vec![Panel {
            id: "TG-1".to_string(),
            name: "Tablero TG-1".to_string(),
            parent: None,
            kind: CircuitKind::ThreePhase,
            feeder: Feeder {
                length_m: 10.0,
                demand_factor: 1.0,
                has_neutral: true,
                overrides: Overrides::default(),
            },
            circuits: vec![],
        }],
    }
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        open: Option<PathBuf>,
        capture: Option<PathBuf>,
        light: bool,
        quick: bool,
    ) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let dark = !light;
        let palette = Palette::new(dark, ACCENTS[0].1);
        theme::apply(&cc.egui_ctx, &palette);
        let mut app = App {
            view: if quick { View::Quick } else { View::Project },
            quick: QuickView::default(),
            project: example::house(),
            path: None,
            dirty: false,
            result: Err(String::new()),
            panel: 0,
            selection: Selection::Circuit(1),
            dark,
            accent: 0,
            palette,
            notice: None,
            done: None,
            capture: capture.map(|path| Capture { path, frames: 0 }),
        };
        if let Some(path) = open {
            app.load(path);
        }
        app.recalculate();
        app
    }

    fn recalculate(&mut self) {
        self.result = calculate(&self.project);
    }

    fn set_project(&mut self, project: Project, path: Option<PathBuf>) {
        self.project = project;
        self.path = path;
        self.dirty = false;
        self.panel = 0;
        self.selection = Selection::Feeder;
        self.recalculate();
    }

    fn load(&mut self, path: PathBuf) {
        let loaded = std::fs::read_to_string(&path)
            .map_err(|e| format!("No se pudo leer {}: {e}", path.display()))
            .and_then(|text| Project::from_json(&text));
        match loaded {
            Ok(project) => {
                self.notice = None;
                self.set_project(project, Some(path));
            }
            Err(message) => self.notice = Some(message),
        }
    }

    fn save_to(&mut self, path: PathBuf) {
        match std::fs::write(&path, self.project.to_json()) {
            Ok(()) => {
                self.path = Some(path);
                self.dirty = false;
                self.notice = None;
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    fn save(&mut self, ask: bool) {
        let target = match (&self.path, ask) {
            (Some(path), false) => Some(path.clone()),
            _ => rfd::FileDialog::new()
                .add_filter("Proyecto de VoltForge", &["json"])
                .set_file_name("proyecto.json")
                .save_file(),
        };
        if let Some(path) = target {
            self.save_to(path);
        }
    }

    /// Escribe la memoria del proyecto o la hoja del cálculo rápido en PDF.
    fn export_pdf(&mut self) {
        let (pdf, name) = match self.view {
            View::Project => (
                self.result
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|r| voltforge_report::project_pdf(&self.project, r)),
                "memoria-de-calculo.pdf",
            ),
            View::Quick => (self.quick.pdf(), "hoja-de-calculo.pdf"),
        };
        let pdf = match pdf {
            Ok(pdf) => pdf,
            Err(e) => {
                self.notice = Some(format!("No se pudo generar el PDF: {e}"));
                return;
            }
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name(name)
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, pdf) {
            Ok(()) => {
                self.notice = None;
                self.done = Some(format!("PDF guardado en {}", path.display()));
            }
            Err(e) => self.notice = Some(format!("No se pudo guardar {}: {e}", path.display())),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Proyecto de VoltForge", &["json"])
            .pick_file()
        {
            self.load(path);
        }
    }

    fn status_color(&self, status: Status) -> Color32 {
        match status {
            Status::Complies => self.palette.ok,
            Status::Fails => self.palette.bad,
            Status::NotEvaluated => self.palette.na,
        }
    }

    fn status_text(&self, status: Status) -> RichText {
        let mark = match status {
            Status::Complies => "✔",
            Status::Fails => "✖",
            Status::NotEvaluated => "—",
        };
        RichText::new(format!("{mark} {}", status.label())).color(self.status_color(status))
    }

    fn caption(&self, text: &str) -> RichText {
        RichText::new(text.to_uppercase())
            .small()
            .color(self.palette.ink2)
    }

    fn muted(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).color(self.palette.ink2)
    }

    fn strong(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).family(theme::semibold())
    }

    fn panel_result(&self) -> Option<PanelResult> {
        let id = &self.project.panels.get(self.panel)?.id;
        self.result
            .as_ref()
            .ok()?
            .panels
            .iter()
            .find(|p| &p.id == id)
            .cloned()
    }

    // ------------------------------------------------------------ barra superior

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚡").color(self.palette.accent).size(16.0));
            ui.label(self.strong("VoltForge").size(15.0));
            ui.add_space(8.0);
            ui.selectable_value(&mut self.view, View::Project, "Proyecto");
            ui.selectable_value(&mut self.view, View::Quick, "Cálculo rápido");
            ui.add_space(8.0);
            ui.menu_button("Archivo", |ui| {
                if ui.button("Nuevo").clicked() {
                    self.view = View::Project;
                    self.set_project(empty_project(), None);
                    ui.close();
                }
                if ui.button("Abrir…").clicked() {
                    ui.close();
                    self.open_dialog();
                }
                if ui.button("Guardar").clicked() {
                    ui.close();
                    self.save(false);
                }
                if ui.button("Guardar como…").clicked() {
                    ui.close();
                    self.save(true);
                }
                ui.separator();
                let export = match self.view {
                    View::Project => "Exportar memoria en PDF…",
                    View::Quick => "Exportar hoja en PDF…",
                };
                if ui.button(export).clicked() {
                    ui.close();
                    self.export_pdf();
                }
                ui.separator();
                if ui.button("Cargar el ejemplo").clicked() {
                    self.view = View::Project;
                    self.set_project(example::house(), None);
                    self.selection = Selection::Circuit(1);
                    ui.close();
                }
            });
            ui.menu_button("Ver", |ui| {
                let mut restyle = false;
                restyle |= ui
                    .radio_value(&mut self.dark, true, "Tema oscuro")
                    .changed();
                restyle |= ui
                    .radio_value(&mut self.dark, false, "Tema claro")
                    .changed();
                ui.separator();
                ui.label(self.muted("Color de acento"));
                for (i, (name, _)) in ACCENTS.iter().enumerate() {
                    restyle |= ui.radio_value(&mut self.accent, i, *name).changed();
                }
                if restyle {
                    self.palette = Palette::new(self.dark, ACCENTS[self.accent].1);
                    theme::apply(ui.ctx(), &self.palette);
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let file = self
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map_or("sin guardar".to_string(), |n| {
                        n.to_string_lossy().into_owned()
                    });
                if self.view == View::Project {
                    ui.label(self.muted(file));
                    ui.label(&self.project.name);
                }
            });
        });
    }

    // ------------------------------------------------------------ barra inferior

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        if self.view == View::Quick {
            ui.horizontal(|ui| {
                ui.label(self.muted(self.quick.summary()));
                if let Some(notice) = &self.notice {
                    ui.label(RichText::new(notice).color(self.palette.bad));
                }
                if let Some(done) = &self.done {
                    ui.label(RichText::new(done).color(self.palette.ok));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(self.muted("Sin aprobación profesional"));
                });
            });
            return;
        }
        ui.horizontal(|ui| {
            match &self.result {
                Ok(r) => {
                    ui.label(
                        RichText::new(format!("{} cumplen", r.complies)).color(self.palette.ok),
                    );
                    ui.label(
                        RichText::new(format!("{} no cumplen", r.fails)).color(self.palette.bad),
                    );
                    ui.label(
                        RichText::new(format!("{} no evaluados", r.not_evaluated))
                            .color(self.palette.na),
                    );
                }
                Err(e) => {
                    ui.label(RichText::new(e).color(self.palette.bad));
                }
            }
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).color(self.palette.bad));
            }
            if let Some(done) = &self.done {
                ui.label(RichText::new(done).color(self.palette.ok));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(self.muted(if self.dirty {
                    "Cambios sin guardar"
                } else if self.path.is_none() {
                    "Sin archivo"
                } else {
                    "Guardado"
                }));
                ui.label(self.muted("Sin aprobación profesional"));
            });
        });
    }

    // ------------------------------------------------------------ panel izquierdo

    fn project_tree(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.label(self.caption("Proyecto"));
        changed |= ui.text_edit_singleline(&mut self.project.name).changed();
        ui.horizontal(|ui| {
            ui.label(self.muted("Suministro"));
            changed |= combo(ui, "sistema", &mut self.project.source.system, &SYSTEMS);
        });
        ui.add_space(4.0);
        for (index, panel) in self.project.panels.iter().enumerate() {
            let selected = self.panel == index && self.selection == Selection::Feeder;
            if ui
                .selectable_label(selected, self.strong(&panel.name))
                .clicked()
            {
                self.panel = index;
                self.selection = Selection::Feeder;
            }
            if self.panel != index {
                continue;
            }
            for (i, circuit) in panel.circuits.iter().enumerate() {
                let text = format!("    {}  {}", circuit.id, circuit.name);
                let selected = self.selection == Selection::Circuit(i);
                if ui.selectable_label(selected, text).clicked() {
                    self.selection = Selection::Circuit(i);
                }
            }
        }

        ui.add_space(12.0);
        ui.label(self.caption("Criterios del proyecto"));
        let muted = |t: &str| RichText::new(t).color(self.palette.ink2);
        let s = &mut self.project.settings;
        egui::Grid::new("criterios")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label(muted("Perfil"));
                ui.label("NOM-001-SEDE-2012");
                ui.end_row();
                ui.label(muted("Caída en derivado"));
                changed |= number(ui, &mut s.branch_drop_limit_percent, 0.1, 0.5, 10.0, " %");
                ui.end_row();
                ui.label(muted("Caída en alimentador"));
                changed |= number(ui, &mut s.feeder_drop_limit_percent, 0.1, 0.5, 10.0, " %");
                ui.end_row();
                ui.label(muted("Caída total"));
                changed |= number(ui, &mut s.total_drop_limit_percent, 0.1, 0.5, 15.0, " %");
                ui.end_row();
                ui.label(muted("Temp. ambiente"));
                changed |= number(ui, &mut s.ambient_c, 1.0, 0.0, 85.0, " °C");
                ui.end_row();
                ui.label(muted("Conductor"));
                changed |= combo(ui, "material", &mut s.material, &MATERIALS);
                ui.end_row();
                ui.label(muted("Aislamiento"));
                changed |= combo(ui, "aislamiento", &mut s.insulation, &INSULATIONS);
                ui.end_row();
                ui.label(muted("Canalización"));
                changed |= combo(ui, "tubo", &mut s.conduit, &CONDUITS);
                ui.end_row();
            });
        changed |= ui
            .checkbox(&mut s.bare_grounding, "Tierra desnuda")
            .changed();
        changed |= ui.checkbox(&mut s.wet_location, "Lugar mojado").changed();
        let mut adjust = s.resistance_temperature_c.is_some();
        if ui
            .checkbox(&mut adjust, "Ajustar resistencia por temperatura")
            .changed()
        {
            s.resistance_temperature_c = adjust.then_some(60.0);
            changed = true;
        }
        if let Some(t) = &mut s.resistance_temperature_c {
            ui.horizontal(|ui| {
                ui.label(muted("Temperatura del conductor"));
                changed |= number(ui, t, 1.0, 20.0, 90.0, " °C");
            });
        }
        changed
    }

    // ------------------------------------------------------------ cuadro de cargas

    fn schedule(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let Some(result) = self.panel_result() else {
            ui.label(self.muted("El proyecto no tiene tableros."));
            return false;
        };
        ui.horizontal(|ui| {
            ui.heading(&result.name);
            ui.label(self.muted(format!(
                "Cuadro de cargas · {} circuitos · {:.0} W conectados · fp {:.2}",
                result.circuits.len(),
                result.connected_w,
                result.power_factor
            )));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let removable = matches!(self.selection, Selection::Circuit(_));
                if ui
                    .add_enabled(removable, egui::Button::new("Eliminar"))
                    .clicked()
                {
                    if let Selection::Circuit(i) = self.selection {
                        self.project.panels[self.panel].circuits.remove(i);
                        self.selection = Selection::Feeder;
                        changed = true;
                    }
                }
                if ui.button("Agregar circuito").clicked() {
                    let circuits = &mut self.project.panels[self.panel].circuits;
                    let next = (1..)
                        .map(|n| format!("C-{n:02}"))
                        .find(|id| circuits.iter().all(|c| &c.id != id));
                    circuits.push(Circuit {
                        id: next.unwrap_or_default(),
                        name: "Circuito nuevo".to_string(),
                        kind: CircuitKind::LineToNeutral,
                        phases: vec![Phase::A],
                        load: Load {
                            power_w: 1000.0,
                            power_factor: 0.9,
                            continuous: false,
                            kind: LoadKind::General,
                        },
                        length_m: 10.0,
                        has_neutral: false,
                        overrides: Overrides::default(),
                    });
                    self.selection = Selection::Circuit(circuits.len() - 1);
                    changed = true;
                }
            });
        });
        let [a, b, c] = result.phase_va;
        let balance = match result.imbalance_percent {
            Some(i) => {
                format!("Fases: A {a:.0} VA · B {b:.0} VA · C {c:.0} VA · desbalance {i:.1} %")
            }
            None => "Fases sin asignar en todos los circuitos".to_string(),
        };
        ui.label(self.muted(balance));
        ui.add_space(6.0);

        let rows: Vec<(Selection, &RunResult)> =
            std::iter::once((Selection::Feeder, &result.feeder))
                .chain(
                    result
                        .circuits
                        .iter()
                        .enumerate()
                        .map(|(i, c)| (Selection::Circuit(i), c)),
                )
                .collect();
        let mono = |t: String| RichText::new(t).monospace();
        let mut clicked = None;
        egui::Frame::new()
            .fill(self.palette.panel)
            .stroke(egui::Stroke::new(1.0, self.palette.line))
            .corner_radius(8)
            .inner_margin(8)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(false)
                    .sense(Sense::click())
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::auto().at_least(44.0))
                    .column(
                        Column::initial(150.0)
                            .at_least(120.0)
                            .resizable(true)
                            .clip(true),
                    )
                    .columns(Column::auto().at_least(40.0), 9)
                    .column(Column::auto().at_least(112.0))
                    .auto_shrink([false, true])
                    .header(24.0, |mut header| {
                        for title in [
                            "ID",
                            "Descripción",
                            "Carga W",
                            "I A",
                            "L m",
                            "Calibre",
                            "Tierra",
                            "Tubo",
                            "ITM",
                            "e %",
                            "Σe %",
                            "Estado",
                        ] {
                            header.col(|ui| {
                                ui.label(self.caption(title));
                            });
                        }
                    })
                    .body(|mut body| {
                        for (selection, item) in &rows {
                            body.row(28.0, |mut row| {
                                row.set_selected(self.selection == *selection);
                                let id = if *selection == Selection::Feeder {
                                    "ALIM"
                                } else {
                                    &item.id
                                };
                                row.col(|ui| {
                                    ui.label(self.muted(id));
                                });
                                row.col(|ui| {
                                    ui.label(&item.name);
                                });
                                row.col(|ui| {
                                    ui.label(mono(format!("{:.0}", item.load_w)));
                                });
                                let cells: [String; 8] = match &item.run {
                                    Some(run) => [
                                        format!("{:.2}", run.nominal_a),
                                        format!("{:.0}", run.length_m),
                                        if run.conductors_per_phase > 1 {
                                            format!("{}×{}", run.conductors_per_phase, run.size.0)
                                        } else {
                                            run.size.0.clone()
                                        },
                                        run.grounding_size.0.clone(),
                                        run.conduit.as_ref().map_or("—".to_string(), |c| {
                                            format!("{} mm", c.metric_designator)
                                        }),
                                        format!("{}P-{:.0}", run.poles, run.breaker_a),
                                        format!("{:.2}", run.drop_percent),
                                        format!("{:.2}", run.cumulative_drop_percent),
                                    ],
                                    None => std::array::from_fn(|_| "—".to_string()),
                                };
                                for cell in cells {
                                    row.col(|ui| {
                                        ui.label(mono(cell));
                                    });
                                }
                                row.col(|ui| {
                                    ui.label(self.status_text(item.status));
                                });
                                if row.response().clicked() {
                                    clicked = Some(*selection);
                                }
                            });
                        }
                    });
            });
        if let Some(selection) = clicked {
            self.selection = selection;
        }

        ui.add_space(8.0);
        for (_, item) in &rows {
            for check in item.checks.iter().filter(|c| c.status != Status::Complies) {
                ui.horizontal_wrapped(|ui| {
                    let id = if item.id == result.id {
                        "Alimentador"
                    } else {
                        &item.id
                    };
                    ui.label(
                        self.strong(format!("{id} · {}", check.name))
                            .color(self.status_color(check.status)),
                    );
                    ui.label(self.muted(&check.detail));
                });
            }
        }
        if let Ok(r) = &self.result {
            for warning in &r.warnings {
                ui.label(RichText::new(format!("Aviso · {warning}")).color(self.palette.na));
            }
        }

        // Procedimiento del elemento seleccionado.
        let selected = rows.iter().find(|(s, _)| *s == self.selection);
        if let Some((selection, item)) = selected {
            if let Some(run) = &item.run {
                let title = match selection {
                    Selection::Feeder => format!("Procedimiento · alimentador de {}", result.name),
                    Selection::Circuit(_) => format!("Procedimiento · {} {}", item.id, item.name),
                };
                ui.add_space(16.0);
                ui.label(caption(&self.palette, &title));
                ui.add_space(2.0);
                ui.scope(|ui| {
                    ui.set_max_width(ui.clip_rect().width() - 4.0);
                    steps_ui(ui, &self.palette, &run.steps);
                });
            }
        }
        changed
    }

    // ------------------------------------------------------------ panel derecho

    fn results(&self, ui: &mut egui::Ui, item: &RunResult) {
        ui.label(self.caption("Resultado"));
        let Some(run) = &item.run else {
            ui.label(self.status_text(item.status));
            for check in &item.checks {
                ui.label(self.muted(&check.detail));
            }
            return;
        };
        let conduit = run.conduit.as_ref().map_or("—".to_string(), |c| {
            format!(
                "{} de {} mm ({}\")",
                c.conduit_type, c.metric_designator, c.trade_size
            )
        });
        let size = if run.conductors_per_phase > 1 {
            format!("{} × {} por fase", run.conductors_per_phase, run.size.0)
        } else {
            run.size.0.clone()
        };
        let rows = [
            ("Corriente nominal", format!("{:.2} A", run.nominal_a)),
            ("Corriente de diseño", format!("{:.2} A", run.design_a)),
            ("Conductor", size),
            (
                "Ampacidad permitida",
                format!("{:.2} A", run.allowable_ampacity_a),
            ),
            ("Puesta a tierra", run.grounding_size.0.clone()),
            ("Tubería", conduit),
            (
                "Interruptor",
                format!("{}P-{:.0} A", run.poles, run.breaker_a),
            ),
            (
                "Caída de tensión",
                format!("{:.2} V · {:.2} %", run.drop_v, run.drop_percent),
            ),
            (
                "Caída acumulada",
                format!("{:.2} %", run.cumulative_drop_percent),
            ),
        ];
        egui::Grid::new("resultado")
            .num_columns(2)
            .spacing([12.0, 5.0])
            .show(ui, |ui| {
                for (name, value) in rows {
                    ui.label(self.muted(name));
                    ui.label(RichText::new(value).monospace());
                    ui.end_row();
                }
            });
        if let Some(governing) = &run.governing {
            let text = if run.forced {
                format!(
                    "Calibre fijado por el usuario. El programa propondría el {} por {governing}.",
                    run.recommended_size.as_ref().map_or("—", |s| s.0.as_str())
                )
            } else {
                format!("Calibre determinado por {governing}.")
            };
            ui.add_space(4.0);
            ui.label(self.muted(text));
        }
        ui.add_space(8.0);
        ui.label(self.caption("Verificaciones"));
        for check in &item.checks {
            ui.horizontal(|ui| {
                ui.label(&check.name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(self.status_text(check.status));
                });
            });
            ui.label(self.muted(&check.detail).small());
        }
    }

    fn detail(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let Some(result) = self.panel_result() else {
            return false;
        };
        let caption = self.caption("Elemento seleccionado");
        let muted = |t: &str| RichText::new(t).color(self.palette.ink2);
        ui.label(caption);
        match self.selection {
            Selection::Circuit(i) if i < self.project.panels[self.panel].circuits.len() => {
                let item = result.circuits.get(i).cloned();
                let mut apply = None;
                {
                    let circuit = &mut self.project.panels[self.panel].circuits[i];
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(egui::TextEdit::singleline(&mut circuit.id).desired_width(60.0))
                            .changed();
                        changed |= ui.text_edit_singleline(&mut circuit.name).changed();
                    });
                    egui::Grid::new("circuito")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(muted("Conexión"));
                            if combo(ui, "conexion", &mut circuit.kind, &KINDS) {
                                circuit.phases.clear();
                                changed = true;
                            }
                            ui.end_row();
                            ui.label(muted("Fases"));
                            ui.horizontal(|ui| {
                                for (label, phase) in
                                    [("A", Phase::A), ("B", Phase::B), ("C", Phase::C)]
                                {
                                    let mut on = circuit.phases.contains(&phase);
                                    if ui.checkbox(&mut on, label).changed() {
                                        if on {
                                            circuit.phases.push(phase);
                                        } else {
                                            circuit.phases.retain(|p| *p != phase);
                                        }
                                        changed = true;
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label(muted("Carga"));
                            changed |=
                                number(ui, &mut circuit.load.power_w, 10.0, 0.0, 5.0e6, " W");
                            ui.end_row();
                            ui.label(muted("Factor de potencia"));
                            changed |=
                                number(ui, &mut circuit.load.power_factor, 0.01, 0.1, 1.0, "");
                            ui.end_row();
                            ui.label(muted("Longitud"));
                            changed |= number(ui, &mut circuit.length_m, 0.5, 0.0, 5000.0, " m");
                            ui.end_row();
                            ui.label(muted("Tipo de carga"));
                            changed |= combo(ui, "tipo-carga", &mut circuit.load.kind, &LOAD_KINDS);
                            ui.end_row();
                            changed |= self_overrides(ui, &mut circuit.overrides, &self.palette);
                        });
                    changed |= ui
                        .checkbox(&mut circuit.load.continuous, "Carga continua")
                        .changed();
                    if circuit.kind != CircuitKind::LineToNeutral {
                        changed |= ui
                            .checkbox(&mut circuit.has_neutral, "Lleva neutro")
                            .changed();
                    }
                    changed |= ui
                        .checkbox(
                            &mut circuit.overrides.nipple,
                            "Tramo de 60 cm o menos (niple)",
                        )
                        .changed();
                    if let Some(run) = item.as_ref().and_then(|i| i.run.as_ref()) {
                        if let (true, Some(recommended)) = (run.forced, &run.recommended_size) {
                            if recommended != &run.size
                                && ui
                                    .button(format!("Aplicar calibre {}", recommended.0))
                                    .clicked()
                            {
                                apply = Some(recommended.clone());
                            }
                        }
                    }
                    if let Some(size) = apply {
                        circuit.overrides.size = Some(size);
                        changed = true;
                    }
                }
                ui.add_space(10.0);
                if let Some(item) = &item {
                    self.results(ui, item);
                }
            }
            _ => {
                let panel = &mut self.project.panels[self.panel];
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(egui::TextEdit::singleline(&mut panel.id).desired_width(60.0))
                        .changed();
                    changed |= ui.text_edit_singleline(&mut panel.name).changed();
                });
                egui::Grid::new("alimentador")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(muted("Conexión"));
                        changed |= combo(ui, "conexion-tablero", &mut panel.kind, &KINDS);
                        ui.end_row();
                        ui.label(muted("Longitud del alimentador"));
                        changed |= number(ui, &mut panel.feeder.length_m, 0.5, 0.0, 5000.0, " m");
                        ui.end_row();
                        ui.label(muted("Factor de demanda"));
                        changed |= number(ui, &mut panel.feeder.demand_factor, 0.01, 0.05, 1.0, "");
                        ui.end_row();
                        changed |= self_overrides(ui, &mut panel.feeder.overrides, &self.palette);
                    });
                changed |= ui
                    .checkbox(&mut panel.feeder.has_neutral, "Lleva neutro")
                    .changed();
                ui.add_space(10.0);
                self.results(ui, &result.feeder);
            }
        }
        changed
    }
}

/// Editor de calibre y conductores por fase, dentro de una rejilla de dos columnas.
fn self_overrides(ui: &mut egui::Ui, ov: &mut Overrides, palette: &Palette) -> bool {
    let mut changed = false;
    let muted = |t: &str| RichText::new(t).color(palette.ink2);
    ui.label(muted("Calibre"));
    let current = ov
        .size
        .as_ref()
        .map_or("Automático".to_string(), |s| s.0.clone());
    egui::ComboBox::from_id_salt("calibre")
        .selected_text(current)
        .width(150.0)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(&mut ov.size, None, "Automático")
                .changed();
            for c in Profile::nom_001_sede_2012()
                .sizes
                .iter()
                .skip_while(|c| c.size != "14")
            {
                let value = Some(Size::new(&c.size));
                changed |= ui.selectable_value(&mut ov.size, value, &c.size).changed();
            }
        });
    ui.end_row();
    ui.label(muted("Conductores por fase"));
    let mut n = ov.conductors_per_phase.unwrap_or(1);
    if ui.add(egui::DragValue::new(&mut n).range(1..=12)).changed() {
        ov.conductors_per_phase = (n > 1).then_some(n);
        changed = true;
    }
    ui.end_row();
    changed
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let p = self.palette;
        let bar = egui::Frame::new()
            .fill(p.panel)
            .inner_margin(egui::Margin::symmetric(14, 8));
        let side = egui::Frame::new().fill(p.panel).inner_margin(14);

        egui::Panel::top("barra")
            .frame(bar)
            .show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("estado")
            .frame(bar)
            .show(ui, |ui| self.status_bar(ui));

        if self.view == View::Quick {
            self.quick.ui(ui, &p);
            self.handle_capture(ui.ctx());
            return;
        }

        let mut changed = false;
        egui::Panel::left("proyecto")
            .frame(side)
            .default_size(264.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| changed |= self.project_tree(ui));
            });
        egui::Panel::right("detalle")
            .frame(side)
            .default_size(330.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| changed |= self.detail(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.bg).inner_margin(16))
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink(false)
                    .show(ui, |ui| changed |= self.schedule(ui));
            });

        if changed {
            self.done = None;
            self.dirty = true;
            self.recalculate();
        }
        self.handle_capture(ui.ctx());
    }
}

impl App {
    /// Guarda una imagen de la ventana y cierra, si se pidió con `--captura`.
    fn handle_capture(&mut self, ctx: &egui::Context) {
        let Some(capture) = &mut self.capture else {
            return;
        };
        capture.frames += 1;
        if capture.frames == 5 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let [w, h] = image.size;
            let saved = image::save_buffer(
                &capture.path,
                image.as_raw(),
                w as u32,
                h as u32,
                image::ColorType::Rgba8,
            );
            if let Err(e) = saved {
                eprintln!("No se pudo guardar la captura: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}
