//! Diagrama unifilar de una instalación radial.
//!
//! Se dibuja solo a partir del proyecto y de su resultado: fuente,
//! alimentadores, tableros y circuitos, con el interruptor, los conductores,
//! la tubería y la caída de tensión de cada tramo. El diagrama es una lista
//! de trazos y textos en coordenadas propias; la interfaz los pinta en
//! pantalla y la memoria los recibe como SVG.

use std::fmt::Write as _;

use voltforge_project::{Project, ProjectResult, RunResult, Status};

/// Ancho de la columna de cada circuito.
const COLUMN: f64 = 116.0;
/// Ancho mínimo de un tablero, para que quepa la etiqueta de su alimentador.
const MIN_PANEL_WIDTH: f64 = 2.0 * COLUMN;
/// Alto del tramo de alimentador, del origen a la barra del tablero.
const FEEDER_HEIGHT: f64 = 120.0;
/// Alto del tramo de circuito, de la barra al símbolo de carga.
const BRANCH_HEIGHT: f64 = 96.0;
const LINE: f64 = 13.0;
const MARGIN: f64 = 24.0;
const SOURCE_HEIGHT: f64 = 36.0;
/// Caracteres que caben en la columna de un circuito.
const COLUMN_CHARS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Text,
    Muted,
    Accent,
    Complies,
    Fails,
    NotEvaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Regular,
    Bold,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// Línea quebrada.
    Path {
        points: Vec<(f64, f64)>,
        width: f64,
        ink: Ink,
    },
    Rect {
        min: (f64, f64),
        size: (f64, f64),
        ink: Ink,
    },
    /// Texto; `pos` es el inicio, centro o fin de la línea base según `anchor`.
    Text {
        pos: (f64, f64),
        text: String,
        size: f64,
        font: Font,
        ink: Ink,
        anchor: Anchor,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagram {
    pub width: f64,
    pub height: f64,
    pub elements: Vec<Element>,
    /// Zonas que corresponden a un tablero o a un circuito, para
    /// seleccionarlos desde el diagrama.
    pub hotspots: Vec<Hotspot>,
}

/// Zona rectangular del diagrama que representa un elemento del proyecto.
#[derive(Debug, Clone, PartialEq)]
pub struct Hotspot {
    pub min: (f64, f64),
    pub size: (f64, f64),
    /// Índice del tablero en el proyecto.
    pub panel: usize,
    /// Índice del circuito, o `None` para el alimentador y la barra.
    pub circuit: Option<usize>,
}

fn status_ink(status: Status) -> Ink {
    match status {
        Status::Complies => Ink::Complies,
        Status::Fails => Ink::Fails,
        Status::NotEvaluated => Ink::NotEvaluated,
    }
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max - 1).collect();
        format!("{}…", cut.trim_end())
    }
}

struct Painter {
    elements: Vec<Element>,
    hotspots: Vec<Hotspot>,
    bottom: f64,
}

impl Painter {
    fn line(&mut self, from: (f64, f64), to: (f64, f64), width: f64) {
        self.bottom = self.bottom.max(from.1).max(to.1);
        self.elements.push(Element::Path {
            points: vec![from, to],
            width,
            ink: Ink::Text,
        });
    }

    fn text(&mut self, pos: (f64, f64), text: String, font: Font, ink: Ink, anchor: Anchor) {
        self.bottom = self.bottom.max(pos.1 + 4.0);
        self.elements.push(Element::Text {
            pos,
            text,
            size: 9.0,
            font,
            ink,
            anchor,
        });
    }

    /// Interruptor: la línea se interrumpe y un arco une los dos extremos.
    fn breaker(&mut self, x: f64, top: f64, bottom: f64) {
        let radius = (bottom - top) / 2.0;
        let center = top + radius;
        let points = (0..=12)
            .map(|i| {
                let angle = std::f64::consts::PI * (f64::from(i) / 12.0 - 0.5);
                (x + radius * angle.cos(), center + radius * angle.sin())
            })
            .collect();
        self.elements.push(Element::Path {
            points,
            width: 1.4,
            ink: Ink::Text,
        });
        for y in [top, bottom] {
            self.elements.push(Element::Path {
                points: vec![(x - 2.5, y), (x + 2.5, y)],
                width: 1.4,
                ink: Ink::Text,
            });
        }
    }

    /// Tramo vertical con interruptor, de `top` a `bottom`.
    fn run(&mut self, x: f64, top: f64, bottom: f64) {
        self.line((x, top), (x, top + 22.0), 1.4);
        self.breaker(x, top + 22.0, top + 40.0);
        self.line((x, top + 40.0), (x, bottom), 1.4);
    }
}

/// Texto de los conductores de un tramo: `4-10, 1-10d`.
fn conductors(item: &RunResult, bare_grounding: bool) -> String {
    match &item.run {
        Some(run) => format!(
            "{}-{}, 1-{}{}",
            run.conductor_count,
            run.size.0,
            run.grounding_size.0,
            if bare_grounding { "d" } else { "" }
        ),
        None => "—".to_string(),
    }
}

fn run_lines(
    item: &RunResult,
    bare_grounding: bool,
    width_chars: usize,
) -> Vec<(String, Font, Ink)> {
    let Some(run) = &item.run else {
        return vec![("No evaluado".to_string(), Font::Regular, Ink::NotEvaluated)];
    };
    let conduit = run.conduit.as_ref().map_or("tubo —".to_string(), |c| {
        format!("{} {} mm", c.conduit_type, c.metric_designator)
    });
    vec![
        (
            format!("{}P-{:.0} A", run.poles, run.breaker_a),
            Font::Mono,
            Ink::Text,
        ),
        (conductors(item, bare_grounding), Font::Mono, Ink::Text),
        (
            shorten(&format!("{conduit} · {:.0} m", run.length_m), width_chars),
            Font::Mono,
            Ink::Muted,
        ),
        (
            format!(
                "e {:.2} · Σ {:.2} %",
                run.drop_percent, run.cumulative_drop_percent
            ),
            Font::Mono,
            status_ink(item.status),
        ),
    ]
}

/// Ancho que ocupa un tablero con sus circuitos y subtableros.
fn panel_width(project: &Project, index: usize) -> f64 {
    let panel = &project.panels[index];
    let circuits = panel.circuits.len() as f64 * COLUMN;
    let subpanels: f64 = children(project, &panel.id)
        .map(|i| panel_width(project, i))
        .sum();
    (circuits + subpanels).max(MIN_PANEL_WIDTH)
}

fn children<'a>(project: &'a Project, id: &'a str) -> impl Iterator<Item = usize> + 'a {
    project
        .panels
        .iter()
        .enumerate()
        .filter(move |(_, p)| p.parent.as_deref() == Some(id))
        .map(|(i, _)| i)
}

fn draw_panel(
    painter: &mut Painter,
    project: &Project,
    result: &ProjectResult,
    index: usize,
    x0: f64,
    top: f64,
) {
    let panel = &project.panels[index];
    let Some(panel_result) = result.panels.iter().find(|p| p.id == panel.id) else {
        return;
    };
    let bare = project.settings.bare_grounding;
    let width = panel_width(project, index);
    let feed_x = x0 + COLUMN / 2.0;
    let bus_y = top + FEEDER_HEIGHT;

    // Alimentador.
    painter.hotspots.push(Hotspot {
        min: (x0, top),
        size: (width, FEEDER_HEIGHT + 22.0),
        panel: index,
        circuit: None,
    });
    painter.run(feed_x, top, bus_y);
    let feeder = &panel_result.feeder;
    let mut y = top + 18.0;
    painter.text(
        (feed_x + 12.0, y),
        format!("Alimentador {}", panel.id),
        Font::Bold,
        status_ink(feeder.status),
        Anchor::Start,
    );
    for (text, font, ink) in run_lines(feeder, bare, 30) {
        y += LINE;
        painter.text((feed_x + 12.0, y), text, font, ink, Anchor::Start);
    }

    // Barra del tablero.
    let bus_end = x0 + width - 10.0;
    painter.line((x0 + 10.0, bus_y), (bus_end, bus_y), 4.0);
    painter.text(
        (bus_end, bus_y - 8.0),
        format!(
            "{} · {:.0} W",
            shorten(&panel.name, 30),
            panel_result.demanded_w
        ),
        Font::Bold,
        Ink::Accent,
        Anchor::End,
    );

    // Circuitos.
    let mut slot = x0;
    for (k, item) in panel_result.circuits.iter().enumerate() {
        painter.hotspots.push(Hotspot {
            min: (slot, bus_y + 6.0),
            size: (COLUMN, BRANCH_HEIGHT + 12.0 + 7.0 * LINE),
            panel: index,
            circuit: Some(k),
        });
        let x = slot + COLUMN / 2.0;
        let end = bus_y + BRANCH_HEIGHT - 12.0;
        painter.run(x, bus_y, end);
        // Carga: flecha hacia abajo.
        painter.elements.push(Element::Path {
            points: vec![
                (x - 5.0, end),
                (x + 5.0, end),
                (x, end + 9.0),
                (x - 5.0, end),
            ],
            width: 1.4,
            ink: Ink::Text,
        });
        let left = slot + 6.0;
        let mut y = bus_y + BRANCH_HEIGHT + 12.0;
        painter.text(
            (left, y),
            item.id.clone(),
            Font::Bold,
            status_ink(item.status),
            Anchor::Start,
        );
        y += LINE;
        painter.text(
            (left, y),
            shorten(&item.name, COLUMN_CHARS),
            Font::Regular,
            Ink::Muted,
            Anchor::Start,
        );
        y += LINE;
        painter.text(
            (left, y),
            format!("{:.0} W", item.load_w),
            Font::Mono,
            Ink::Text,
            Anchor::Start,
        );
        for (text, font, ink) in run_lines(item, bare, COLUMN_CHARS) {
            y += LINE;
            painter.text((left, y), text, font, ink, Anchor::Start);
        }
        slot += COLUMN;
    }

    // Subtableros, a la derecha de los circuitos.
    for child in children(project, &panel.id) {
        let w = panel_width(project, child);
        draw_panel(painter, project, result, child, slot, bus_y);
        slot += w;
    }
}

/// Diagrama unifilar del proyecto.
pub fn single_line(project: &Project, result: &ProjectResult) -> Diagram {
    let mut painter = Painter {
        elements: Vec::new(),
        hotspots: Vec::new(),
        bottom: 0.0,
    };
    let roots: Vec<usize> = project
        .panels
        .iter()
        .enumerate()
        .filter(|(_, p)| p.parent.is_none())
        .map(|(i, _)| i)
        .collect();
    let total: f64 = roots.iter().map(|&i| panel_width(project, i)).sum();

    // Fuente: un recuadro con el nombre y la tensión, del que salen los alimentadores.
    let system = &project.source.system;
    let voltage = match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(ll), Some(ln)) => format!("{ll:.0}/{ln:.0} V"),
        (Some(ll), None) => format!("{ll:.0} V"),
        (None, Some(ln)) => format!("{ln:.0} V"),
        (None, None) => String::new(),
    };
    let source_top = MARGIN;
    let source_bottom = source_top + SOURCE_HEIGHT;
    let first_feed = MARGIN + COLUMN / 2.0;
    let box_width = 170.0;
    painter.elements.push(Element::Rect {
        min: (first_feed - box_width / 2.0 + 30.0, source_top),
        size: (box_width, SOURCE_HEIGHT),
        ink: Ink::Text,
    });
    let box_center = first_feed + 30.0;
    painter.text(
        (box_center, source_top + 15.0),
        shorten(&project.source.name, 26),
        Font::Bold,
        Ink::Text,
        Anchor::Middle,
    );
    painter.text(
        (box_center, source_top + 28.0),
        voltage,
        Font::Mono,
        Ink::Muted,
        Anchor::Middle,
    );

    // Barra de la fuente cuando hay más de un tablero principal.
    let top = source_bottom + 14.0;
    painter.line((first_feed, source_bottom), (first_feed, top), 1.4);
    if roots.len() > 1 {
        let mut x = MARGIN;
        let last = roots
            .iter()
            .take(roots.len() - 1)
            .map(|&i| panel_width(project, i))
            .sum::<f64>();
        painter.line((first_feed, top), (MARGIN + last + COLUMN / 2.0, top), 2.0);
        for &i in &roots {
            draw_panel(&mut painter, project, result, i, x, top);
            x += panel_width(project, i);
        }
    } else if let Some(&i) = roots.first() {
        draw_panel(&mut painter, project, result, i, MARGIN, top);
    }

    let mut legend_y = painter.bottom + 14.0;
    for line in [
        "Calibres en AWG o kcmil · d: conductor de puesta a tierra desnudo · W: carga demandada del tablero",
        "e: caída de tensión del tramo, % · Σ: caída acumulada desde la fuente, %",
    ] {
        legend_y += LINE;
        painter.text(
            (MARGIN, legend_y),
            line.to_string(),
            Font::Regular,
            Ink::Muted,
            Anchor::Start,
        );
    }

    Diagram {
        width: (MARGIN * 2.0 + total).max(MARGIN * 2.0 + 600.0),
        height: legend_y + MARGIN,
        elements: painter.elements,
        hotspots: painter.hotspots,
    }
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl Diagram {
    /// El diagrama como SVG, con colores para impresión sobre fondo blanco.
    pub fn to_svg(&self) -> String {
        let color = |ink: Ink| match ink {
            Ink::Text => "#1b222c",
            Ink::Muted => "#5d6877",
            Ink::Accent => "#b97a00",
            Ink::Complies => "#1f8a5b",
            Ink::Fails => "#b8401a",
            Ink::NotEvaluated => "#7a5a00",
        };
        let mut out = String::new();
        let _ = writeln!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}">"#,
            w = self.width,
            h = self.height
        );
        for element in &self.elements {
            match element {
                Element::Path { points, width, ink } => {
                    let pts: Vec<String> = points
                        .iter()
                        .map(|(x, y)| format!("{x:.1},{y:.1}"))
                        .collect();
                    let _ = writeln!(
                        out,
                        r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{width}" stroke-linejoin="round" stroke-linecap="round"/>"#,
                        pts.join(" "),
                        color(*ink)
                    );
                }
                Element::Rect { min, size, ink } => {
                    let _ = writeln!(
                        out,
                        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="3" fill="none" stroke="{}" stroke-width="1.4"/>"#,
                        min.0,
                        min.1,
                        size.0,
                        size.1,
                        color(*ink)
                    );
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
                        Font::Regular => "IBM Plex Sans",
                        Font::Bold => "IBM Plex Sans SmBld",
                        Font::Mono => "IBM Plex Mono",
                    };
                    let anchor = match anchor {
                        Anchor::Start => "start",
                        Anchor::Middle => "middle",
                        Anchor::End => "end",
                    };
                    let _ = writeln!(
                        out,
                        r#"<text x="{:.1}" y="{:.1}" font-family="{family}" font-size="{size}" fill="{}" text-anchor="{anchor}">{}</text>"#,
                        pos.0,
                        pos.1,
                        color(*ink),
                        xml(text)
                    );
                }
            }
        }
        out.push_str("</svg>\n");
        out
    }
}
