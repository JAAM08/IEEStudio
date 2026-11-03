//! Paleta de comandos (Ctrl+K): escribir lo que se quiere hacer o el
//! circuito al que se quiere ir, como en los programas de CAD.
//!
//! La paleta solo arma la lista y dice qué se eligió; quien la abre ejecuta
//! el comando.

use eframe::egui::{self, Key, RichText};
use egui_phosphor::regular as icon;

use crate::state::{Doc, Selection};
use crate::theme::Palette;
use crate::widgets::{muted, strong};
use crate::workspace::{Tab, Workspace};

/// Lo que se puede hacer desde la paleta.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Home,
    New,
    Open,
    Save,
    Example,
    ExportPdf,
    ExportExcel,
    ImportExcel,
    ExportDxf,
    Undo,
    Redo,
    AddCircuit,
    AddPanel,
    Duplicate,
    Remove,
    Balance,
    Quick,
    Panel(Tab),
    Workspace(Workspace),
    ResetWorkspace,
    ToggleTheme,
    Select(Selection),
}

/// Un renglón de la paleta: el comando, su texto, su ícono, palabras para
/// encontrarlo y su atajo.
struct Entry {
    command: Command,
    label: String,
    glyph: &'static str,
    keywords: &'static str,
    shortcut: &'static str,
}

fn entry(
    command: Command,
    glyph: &'static str,
    label: impl Into<String>,
    keywords: &'static str,
    shortcut: &'static str,
) -> Entry {
    Entry {
        command,
        label: label.into(),
        glyph,
        keywords,
        shortcut,
    }
}

/// Todos los comandos disponibles con el proyecto como está.
fn entries(doc: &Doc) -> Vec<Entry> {
    let mut list = vec![
        entry(
            Command::Quick,
            icon::CALCULATOR,
            "Cálculo rápido de un circuito",
            "circuito suelto caida",
            "F8",
        ),
        entry(
            Command::AddCircuit,
            icon::PLUS,
            "Agregar circuito",
            "nuevo circuito c",
            "",
        ),
        entry(
            Command::AddPanel,
            icon::SQUARES_FOUR,
            "Agregar tablero",
            "nuevo tablero t subtablero",
            "",
        ),
        entry(
            Command::Balance,
            icon::SCALES,
            "Balancear fases del tablero",
            "desbalance fases",
            "",
        ),
        entry(
            Command::ExportPdf,
            icon::FILE_PDF,
            "Exportar memoria en PDF",
            "memoria pdf imprimir",
            "Ctrl+E",
        ),
        entry(
            Command::ExportExcel,
            icon::FILE_XLS,
            "Exportar cuadro de cargas a Excel",
            "excel xlsx",
            "",
        ),
        entry(
            Command::ImportExcel,
            icon::FILE_XLS,
            "Importar cuadro de cargas de Excel",
            "excel xlsx importar",
            "",
        ),
        entry(
            Command::ExportDxf,
            icon::FLOW_ARROW,
            "Exportar unifilar a DXF",
            "autocad cad dxf unifilar",
            "",
        ),
        entry(
            Command::Save,
            icon::FLOPPY_DISK,
            "Guardar",
            "guardar archivo",
            "Ctrl+S",
        ),
        entry(
            Command::Open,
            icon::FOLDER_OPEN,
            "Abrir proyecto",
            "abrir archivo",
            "Ctrl+O",
        ),
        entry(
            Command::New,
            icon::FILE_PLUS,
            "Nuevo proyecto",
            "nuevo",
            "Ctrl+N",
        ),
        entry(
            Command::Example,
            icon::HOUSE,
            "Abrir el proyecto de ejemplo",
            "ejemplo casa",
            "",
        ),
        entry(
            Command::Home,
            icon::HOUSE_LINE,
            "Inicio",
            "inicio bienvenida recientes",
            "",
        ),
        entry(
            Command::Undo,
            icon::ARROW_COUNTER_CLOCKWISE,
            "Deshacer",
            "deshacer",
            "Ctrl+Z",
        ),
        entry(
            Command::Redo,
            icon::ARROW_CLOCKWISE,
            "Rehacer",
            "rehacer",
            "Ctrl+Y",
        ),
        entry(
            Command::ResetWorkspace,
            icon::LAYOUT,
            "Restablecer el espacio de trabajo",
            "paneles acomodo",
            "",
        ),
        entry(
            Command::ToggleTheme,
            icon::SUN,
            "Cambiar tema claro u oscuro",
            "tema claro oscuro colores",
            "",
        ),
    ];
    if matches!(doc.selection, Selection::Circuit(..)) {
        list.push(entry(
            Command::Duplicate,
            icon::COPY,
            "Duplicar circuito",
            "copiar",
            "Ctrl+D",
        ));
        list.push(entry(
            Command::Remove,
            icon::TRASH,
            "Eliminar circuito",
            "borrar quitar",
            "Supr",
        ));
    }
    for tab in Tab::ALL {
        list.push(entry(
            Command::Panel(tab),
            tab.icon(),
            format!("Panel: {}", tab.title()),
            "panel ventana abrir",
            tab.shortcut(),
        ));
    }
    for ws in Workspace::ALL {
        list.push(entry(
            Command::Workspace(ws),
            icon::LAYOUT,
            format!("Espacio de trabajo: {}", ws.title()),
            "espacio",
            "",
        ));
    }
    // Ir a un tablero o a un circuito por su ID o su descripción.
    for (i, panel) in doc.project.panels.iter().enumerate() {
        list.push(entry(
            Command::Select(Selection::Feeder(i)),
            icon::SQUARES_FOUR,
            format!("Ir a {} · {}", panel.id, panel.name),
            "ir tablero alimentador",
            "",
        ));
        for (c, circuit) in panel.circuits.iter().enumerate() {
            list.push(entry(
                Command::Select(Selection::Circuit(i, c)),
                icon::PLUG,
                format!("Ir a {} · {} ({})", circuit.id, circuit.name, panel.id),
                "ir circuito",
                "",
            ));
        }
    }
    list
}

/// Minúsculas y sin acentos, para que «caida» encuentre «Caída».
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// Cada palabra de la búsqueda debe aparecer en el texto o en sus palabras
/// clave.
fn matches(entry: &Entry, query: &str) -> bool {
    let haystack = fold(&format!("{} {}", entry.label, entry.keywords));
    fold(query)
        .split_whitespace()
        .all(|word| haystack.contains(word))
}

#[derive(Default)]
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
    /// La paleta se acaba de abrir: el cuadro de texto toma el foco.
    focus: bool,
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.query.clear();
        self.selected = 0;
        self.focus = self.open;
    }

    /// Dibuja la paleta. Devuelve el comando elegido, si lo hay.
    pub fn show(&mut self, ctx: &egui::Context, p: &Palette, doc: &Doc) -> Option<Command> {
        if !self.open {
            return None;
        }
        let all = entries(doc);
        let found: Vec<&Entry> = all.iter().filter(|e| matches(e, &self.query)).collect();
        self.selected = self.selected.min(found.len().saturating_sub(1));
        let (up, down, enter) = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::Enter),
            )
        });
        if down && self.selected + 1 < found.len() {
            self.selected += 1;
        }
        if up {
            self.selected = self.selected.saturating_sub(1);
        }
        let mut chosen = None;
        let modal = egui::Modal::new(egui::Id::new("paleta-de-comandos")).show(ctx, |ui| {
            ui.set_width(560.0);
            let edit = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(format!(
                        "{}  Escribe un comando o un circuito: «exportar», «C-04», «caída»…",
                        icon::MAGNIFYING_GLASS
                    ))
                    .desired_width(f32::INFINITY)
                    .margin(egui::Margin::symmetric(8, 6)),
            );
            if self.focus {
                edit.request_focus();
                self.focus = false;
            }
            if edit.changed() {
                self.selected = 0;
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if found.is_empty() {
                        ui.label(muted(p, "Nada coincide con la búsqueda."));
                    }
                    for (k, e) in found.iter().enumerate() {
                        let selected = k == self.selected;
                        let row = ui
                            .horizontal(|ui| {
                                ui.set_min_height(26.0);
                                ui.set_width(ui.available_width());
                                if selected {
                                    ui.painter().rect_filled(ui.max_rect(), 3.0, p.selected);
                                }
                                ui.add_space(6.0);
                                ui.label(RichText::new(e.glyph).color(if selected {
                                    p.accent_text
                                } else {
                                    p.ink2
                                }));
                                ui.label(if selected {
                                    strong(&e.label)
                                } else {
                                    RichText::new(&e.label)
                                });
                                if !e.shortcut.is_empty() {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(muted(p, e.shortcut).size(11.0));
                                        },
                                    );
                                }
                            })
                            .response
                            .interact(egui::Sense::click());
                        if selected && (up || down) {
                            row.scroll_to_me(None);
                        }
                        if row.clicked() {
                            chosen = Some(e.command.clone());
                        }
                        // Solo el puntero en movimiento cambia el renglón: uno
                        // quieto no le gana a las flechas.
                        let moved = ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO);
                        if row.hovered() && moved {
                            self.selected = k;
                        }
                    }
                });
            ui.add_space(4.0);
            ui.label(muted(p, "↑ ↓ para elegir · Intro para hacerlo · Esc para cerrar").size(11.0));
        });
        if enter {
            chosen = found.get(self.selected).map(|e| e.command.clone());
        }
        if chosen.is_some() || modal.should_close() {
            self.open = false;
        }
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(query: &str) -> Vec<String> {
        let doc = Doc::new(ieestudio_project::example::house(), None);
        entries(&doc)
            .into_iter()
            .filter(|e| matches(e, query))
            .map(|e| e.label)
            .collect()
    }

    #[test]
    fn finds_commands_without_accents() {
        assert!(search("calculo rapido")
            .iter()
            .any(|l| l == "Cálculo rápido de un circuito"));
        assert!(search("exportar pdf")
            .iter()
            .any(|l| l == "Exportar memoria en PDF"));
    }

    #[test]
    fn finds_circuits_by_id_and_name() {
        let found = search("c-04");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("Ir a C-04 · Contactos sala"));
        assert!(search("bomba").iter().any(|l| l.contains("C-06")));
    }
}
