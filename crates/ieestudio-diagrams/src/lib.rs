//! Diagrama unifilar de una instalación radial.
//!
//! Se dibuja solo a partir del proyecto y de su resultado, con el formato de
//! un plano eléctrico: la acometida y el medidor, cada alimentador con su
//! interruptor, sus conductores (fase, neutro y puesta a tierra), tubería,
//! corriente, longitud y caída de tensión; los transformadores; cada tablero
//! en un recuadro punteado con su carga instalada y demandada; y cada
//! circuito con el símbolo de su carga. Los subtableros cuelgan debajo del
//! tablero que los alimenta.
//!
//! Hay dos modos. **Revisión** colorea cada tramo según cumple o no, para
//! trabajar en la aplicación. **Plano** va en blanco y negro con la hoja
//! completa (simbología, notas y cajetín), para la memoria, el PDF y el DXF.
//!
//! El diagrama es una lista de trazos y textos en coordenadas propias; la
//! interfaz los pinta en pantalla, la memoria los recibe como SVG y AutoCAD
//! como DXF.

use std::fmt::Write as _;

use ieestudio_core::{CircuitKind, Insulation, Material, System};
use ieestudio_project::fault::FaultOrigin;
use ieestudio_project::positions::panel_positions;
use ieestudio_project::{
    insulation_name, size_label, BreakerData, LoadKind, Overrides, PanelResult, Project,
    ProjectResult, RunResult, Status,
};

/// Ancho de la columna de cada circuito.
const COLUMN: f64 = 138.0;
/// Margen entre el borde del tablero y su primera columna.
const PAD: f64 = 18.0;
/// Columnas por fila; con más circuitos la barra sigue en otra fila.
const MAX_COLUMNS: usize = 12;
/// Ancho mínimo de un tablero: caben su alimentador y su encabezado.
const MIN_PANEL_WIDTH: f64 = 480.0;
const LINE: f64 = 12.0;
const TEXT: f64 = 9.0;
const MARGIN: f64 = 28.0;
/// Ancho de la columna de la hoja con la simbología, las notas y el cajetín.
const SIDE: f64 = 330.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Text,
    Muted,
    /// Tableros y subtableros.
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
    /// Línea quebrada; punteada en los límites de los tableros.
    Path {
        points: Vec<(f64, f64)>,
        width: f64,
        ink: Ink,
        dashed: bool,
    },
    Rect {
        min: (f64, f64),
        size: (f64, f64),
        ink: Ink,
        dashed: bool,
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

/// Cómo se dibuja el unifilar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Colores según cumple, no cumple o sin evaluar, para la aplicación.
    #[default]
    Review,
    /// Blanco y negro, como plano, para imprimir y para CAD.
    Plan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    /// Hoja completa: marco, simbología, notas y cajetín.
    pub sheet: bool,
}

impl Options {
    pub const REVIEW: Options = Options {
        mode: Mode::Review,
        sheet: false,
    };
    pub const PLAN: Options = Options {
        mode: Mode::Plan,
        sheet: true,
    };
}

/// Símbolos del diagrama, que se repiten en la simbología de la hoja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    Breaker,
    Meter,
    Transformer,
    Panel,
    Load(LoadKind),
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

/// Tensión de un sistema como en los planos: `220/127 V`.
fn voltage_label(system: &System) -> String {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(ll), Some(ln)) => format!("{ll:.0}/{ln:.0} V"),
        (Some(ll), None) => format!("{ll:.0} V"),
        (None, Some(ln)) => format!("{ln:.0} V"),
        (None, None) => String::new(),
    }
}

/// Fases e hilos de un sistema de suministro: `3F-4H`, `1F-3H`…
fn system_wires(system: &System) -> &'static str {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        (None, Some(_)) => "1F-2H",
        (Some(_), None) => "3F-3H",
        // Monofásico de tres hilos: la tensión entre fases es el doble.
        (Some(ll), Some(ln)) if (ll / ln - 2.0).abs() < 0.1 => "1F-3H",
        (Some(_), Some(_)) => "3F-4H",
        (None, None) => "",
    }
}

/// Fases e hilos de una conexión, sin contar el de puesta a tierra.
fn kind_wires(kind: CircuitKind, neutral: bool) -> &'static str {
    match (kind, neutral) {
        (CircuitKind::LineToNeutral, _) => "1F-2H",
        (CircuitKind::LineToLine, false) => "2F-2H",
        (CircuitKind::LineToLine, true) => "2F-3H",
        (CircuitKind::ThreePhase, false) => "3F-3H",
        (CircuitKind::ThreePhase, true) => "3F-4H",
    }
}

fn material_symbol(material: Material) -> &'static str {
    match material {
        Material::Copper => "Cu",
        Material::Aluminum => "Al",
    }
}

/// Sistema de las barras de un tablero, del modelo de conexiones.
fn bus_system(project: &Project, panel: usize) -> System {
    ieestudio_project::connection::bus_system(project, panel)
}

/// Sistema del que se alimenta un tablero: el de su tablero padre, o el de la
/// fuente.
fn upstream_system(project: &Project, panel: usize) -> System {
    ieestudio_project::connection::feeder_system(project, panel)
}

/// Un dato de placa con su unidad, o `missing`: el unifilar no muestra un
/// valor que no se capturó.
fn known(value: Option<f64>, unit: &str, missing: &str) -> String {
    match value {
        Some(v) => format!("{v} {unit}"),
        None => missing.to_string(),
    }
}

fn children(project: &Project, panel: usize) -> Vec<usize> {
    let id = &project.panels[panel].id;
    (0..project.panels.len())
        .filter(|&k| project.panels[k].parent.as_ref() == Some(id))
        .collect()
}

struct Painter {
    elements: Vec<Element>,
    hotspots: Vec<Hotspot>,
    /// Extensión de lo dibujado, para el tamaño de la hoja.
    right: f64,
    bottom: f64,
    mode: Mode,
    /// Símbolos usados, en el orden en que aparecen, para la simbología.
    used: Vec<Symbol>,
}

impl Painter {
    fn ink(&self, ink: Ink) -> Ink {
        match (self.mode, ink) {
            (Mode::Plan, Ink::Complies | Ink::Fails | Ink::NotEvaluated) => Ink::Text,
            _ => ink,
        }
    }

    fn reach(&mut self, x: f64, y: f64) {
        self.right = self.right.max(x);
        self.bottom = self.bottom.max(y);
    }

    fn path(&mut self, points: Vec<(f64, f64)>, width: f64, dashed: bool) {
        for &(x, y) in &points {
            self.reach(x, y);
        }
        self.elements.push(Element::Path {
            points,
            width,
            ink: Ink::Text,
            dashed,
        });
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), width: f64) {
        self.path(vec![from, to], width, false);
    }

    fn rect(&mut self, min: (f64, f64), size: (f64, f64), dashed: bool) {
        self.reach(min.0 + size.0, min.1 + size.1);
        self.elements.push(Element::Rect {
            min,
            size,
            ink: Ink::Text,
            dashed,
        });
    }

    fn text_sized(
        &mut self,
        pos: (f64, f64),
        text: String,
        size: f64,
        font: Font,
        ink: Ink,
        anchor: Anchor,
    ) {
        let chars = text.chars().count() as f64;
        let width = chars
            * size
            * match font {
                Font::Mono => 0.6,
                _ => 0.55,
            };
        let right = match anchor {
            Anchor::Start => pos.0 + width,
            Anchor::Middle => pos.0 + width / 2.0,
            Anchor::End => pos.0,
        };
        self.reach(right, pos.1 + 4.0);
        let ink = self.ink(ink);
        self.elements.push(Element::Text {
            pos,
            text,
            size,
            font,
            ink,
            anchor,
        });
    }

    fn text(&mut self, pos: (f64, f64), text: String, font: Font, ink: Ink, anchor: Anchor) {
        self.text_sized(pos, text, TEXT, font, ink, anchor);
    }

    /// Renglones de texto hacia abajo desde `pos`. Devuelve dónde terminan.
    fn lines(&mut self, pos: (f64, f64), lines: Vec<(String, Font, Ink)>) -> f64 {
        let mut y = pos.1;
        for (text, font, ink) in lines {
            self.text((pos.0, y), text, font, ink, Anchor::Start);
            y += LINE;
        }
        y - LINE
    }

    fn uses(&mut self, symbol: Symbol) {
        if !self.used.contains(&symbol) {
            self.used.push(symbol);
        }
    }

    fn circle(&mut self, center: (f64, f64), radius: f64) {
        let points = (0..=32)
            .map(|i| {
                let angle = std::f64::consts::TAU * f64::from(i) / 32.0;
                (
                    center.0 + radius * angle.cos(),
                    center.1 + radius * angle.sin(),
                )
            })
            .collect();
        self.path(points, 1.2, false);
    }

    /// Interruptor termomagnético entre `top` y `top + 18`: dos contactos y
    /// el arco que los une.
    fn breaker(&mut self, x: f64, top: f64) {
        self.uses(Symbol::Breaker);
        let bottom = top + 18.0;
        let radius = 9.0;
        let center = top + radius;
        let points = (0..=12)
            .map(|i| {
                let angle = std::f64::consts::PI * (f64::from(i) / 12.0 - 0.5);
                (x + radius * angle.cos(), center + radius * angle.sin())
            })
            .collect();
        // Figura 4.2.14: la línea se abre y un arco une los dos extremos.
        let _ = bottom;
        self.path(points, 1.3, false);
    }

    /// Símbolo de una carga o de un tablero con su centro en `c`; ocupa unos
    /// 24 puntos de alto.
    fn symbol(&mut self, symbol: Symbol, c: (f64, f64)) {
        self.uses(symbol);
        let (x, y) = c;
        let letters = |p: &mut Painter, text: &str, size: f64| {
            p.text_sized(
                (x, y + size * 0.36),
                text.to_string(),
                size,
                Font::Bold,
                Ink::Text,
                Anchor::Middle,
            );
        };
        match symbol {
            // Figura 4.2.52, salida de lámpara incandescente: circunferencia
            // con cuatro rayos en diagonal.
            Symbol::Load(LoadKind::Lighting) => {
                self.circle(c, 7.0);
                for (dx, dy) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                    self.line(
                        (x + dx * 5.6, y + dy * 5.6),
                        (x + dx * 10.0, y + dy * 10.0),
                        1.2,
                    );
                }
            }
            // Figura 4.2.27, receptáculo sencillo: circunferencia con dos
            // diagonales.
            Symbol::Load(LoadKind::Receptacles) => {
                self.circle(c, 9.0);
                self.line((x - 7.5, y + 3.0), (x + 3.0, y - 7.5), 1.2);
                self.line((x - 3.0, y + 7.5), (x + 7.5, y - 3.0), 1.2);
            }
            // Figura 4.2.67.
            Symbol::Load(LoadKind::Motor) => {
                self.circle(c, 11.0);
                letters(self, "M", 11.0);
            }
            // Figura 4.2.60, salida para propósito especial: circunferencia
            // con un triángulo y las letras de su función a un lado.
            Symbol::Load(
                kind @ (LoadKind::AirConditioning
                | LoadKind::Appliance
                | LoadKind::KitchenEquipment),
            ) => {
                self.circle(c, 11.0);
                self.path(
                    vec![
                        (x, y - 9.5),
                        (x + 8.2, y + 4.8),
                        (x - 8.2, y + 4.8),
                        (x, y - 9.5),
                    ],
                    1.2,
                    false,
                );
                let text = match kind {
                    LoadKind::AirConditioning => "AA",
                    LoadKind::Appliance => "AF",
                    _ => "EC",
                };
                self.text_sized(
                    (x + 13.0, y + 11.0),
                    text.to_string(),
                    7.0,
                    Font::Bold,
                    Ink::Text,
                    Anchor::Start,
                );
            }
            Symbol::Load(LoadKind::General) => {
                self.path(
                    vec![
                        (x - 6.0, y - 1.0),
                        (x + 6.0, y - 1.0),
                        (x, y + 10.0),
                        (x - 6.0, y - 1.0),
                    ],
                    1.2,
                    false,
                );
            }
            Symbol::Panel => {
                self.rect((x - 15.0, y - 8.0), (30.0, 16.0), false);
                self.line((x - 15.0, y + 8.0), (x + 15.0, y - 8.0), 1.2);
            }
            Symbol::Meter => {
                self.rect((x - 10.0, y - 10.0), (20.0, 20.0), false);
                letters(self, "M", 11.0);
            }
            // Figuras 4.2.68 y 4.3.6: dos devanados enfrentados. El primario
            // arriba y el secundario abajo, porque el diagrama corre de
            // arriba hacia abajo.
            Symbol::Transformer => {
                for (base, sign) in [(y - 4.0, -1.0), (y + 4.0, 1.0)] {
                    for k in [-1.0, 0.0, 1.0] {
                        let cx = x + k * 8.0;
                        let points = (0..=10)
                            .map(|i| {
                                let angle = std::f64::consts::PI * f64::from(i) / 10.0;
                                (cx - 4.0 * angle.cos(), base + sign * 4.0 * angle.sin())
                            })
                            .collect();
                        self.path(points, 1.2, false);
                    }
                    // La terminal sale del centro del devanado.
                    self.line((x, base + sign * 4.0), (x, base + sign * 11.0), 1.2);
                }
            }
            Symbol::Breaker => {
                self.line((x, y - 13.0), (x, y - 9.0), 1.2);
                self.breaker(x, y - 9.0);
                self.line((x, y + 9.0), (x, y + 13.0), 1.2);
            }
        }
    }

    /// Medio alto de un símbolo, para que la línea llegue a su borde.
    fn symbol_half(symbol: Symbol) -> f64 {
        match symbol {
            Symbol::Load(LoadKind::Lighting) => 12.0,
            Symbol::Load(LoadKind::Receptacles) => 10.0,
            Symbol::Load(LoadKind::General) => 1.0,
            Symbol::Panel => 8.0,
            _ => 11.0,
        }
    }
}

/// Lo que cuelga de una columna de un tablero.
#[derive(Clone, Copy)]
enum Slot {
    Circuit(usize),
    Subpanel(usize),
}

fn slots(project: &Project, panel: usize) -> Vec<Slot> {
    let mut out: Vec<Slot> = (0..project.panels[panel].circuits.len())
        .map(Slot::Circuit)
        .collect();
    out.extend(children(project, panel).into_iter().map(Slot::Subpanel));
    out
}

/// Ancho que ocupa un tablero con sus circuitos y subtableros.
fn panel_width(project: &Project, panel: usize) -> f64 {
    let slots = slots(project, panel);
    let in_row = slots.len().clamp(1, MAX_COLUMNS);
    let own = (2.0 * PAD + in_row as f64 * COLUMN).max(MIN_PANEL_WIDTH);
    // Cada subtablero empieza bajo su columna y se extiende a la derecha.
    slots
        .iter()
        .enumerate()
        .filter_map(|(k, s)| match s {
            Slot::Subpanel(child) => {
                let column = k % MAX_COLUMNS;
                Some(column as f64 * COLUMN + panel_width(project, *child))
            }
            Slot::Circuit(_) => None,
        })
        .fold(own, f64::max)
}

/// Datos que acompañan un tramo en el diagrama.
struct RunContext<'a> {
    /// Primer renglón del bloque: el circuito o el alimentador.
    label: String,
    item: &'a RunResult,
    overrides: &'a Overrides,
}

/// Bloque de conductores de un tramo (requisito I.3): tubería, conductores
/// de fase, neutro y puesta a tierra con su material, corriente, longitud y
/// caída de tensión.
fn conductor_lines(project: &Project, ctx: &RunContext) -> Vec<(String, Font, Ink)> {
    let Some(run) = &ctx.item.run else {
        return vec![
            (ctx.label.clone(), Font::Bold, status_ink(ctx.item.status)),
            ("Sin evaluar".to_string(), Font::Regular, Ink::NotEvaluated),
        ];
    };
    let settings = &project.settings;
    let material = material_symbol(ctx.overrides.material.unwrap_or(settings.material));
    let insulation: Insulation = ctx.overrides.insulation.unwrap_or(settings.insulation);
    let insulation = insulation_name(insulation);
    let size = size_label(&run.size);
    let phases = run.poles * run.conductors_per_phase;
    let neutral = run.conductor_count.saturating_sub(phases);
    let mut lines = vec![(
        format!("{} · L = {} m", ctx.label, run.length_m),
        Font::Bold,
        status_ink(ctx.item.status),
    )];
    let conduit = match &run.conduit {
        Some(c) if c.count > 1 => format!(
            "{} TUBOS {} {} ({}\")",
            c.count, c.conduit_type, c.metric_designator, c.trade_size
        ),
        Some(c) => format!(
            "TUBO {} {} ({}\")",
            c.conduit_type, c.metric_designator, c.trade_size
        ),
        None => "TUBO —".to_string(),
    };
    lines.push((conduit, Font::Mono, Ink::Text));
    let per_phase = if run.conductors_per_phase > 1 {
        format!(" ({} por fase)", run.conductors_per_phase)
    } else {
        String::new()
    };
    lines.push((
        format!(
            "{phases}-{size} {}F{per_phase} {material} {insulation}",
            run.poles
        ),
        Font::Mono,
        Ink::Text,
    ));
    if neutral > 0 {
        lines.push((
            format!("{neutral}-{size} {neutral}N {material} {insulation}"),
            Font::Mono,
            Ink::Text,
        ));
    }
    let bare = if settings.bare_grounding {
        "desnudo".to_string()
    } else {
        insulation.to_string()
    };
    lines.push((
        format!("1-{} CPT Cu {bare}", size_label(&run.grounding_size)),
        Font::Mono,
        Ink::Text,
    ));
    lines.push((format!("In: {:.2} A", run.nominal_a), Font::Mono, Ink::Text));
    lines.push((
        format!(
            "%e: {:.2} (Σ {:.2})",
            run.drop_percent, run.cumulative_drop_percent
        ),
        Font::Mono,
        status_ink(ctx.item.status),
    ));
    lines
}

/// Renglones del interruptor de un tramo (requisito I.4): polos y amperes;
/// con datos de catálogo, amperes de marco y de disparo, modelo, capacidad
/// interruptiva, marca e intervalo de ajuste. `board_ka` es la capacidad
/// interruptiva del tablero donde está, si el interruptor no tiene la suya.
fn breaker_lines(item: &RunResult, data: &BreakerData, board_ka: Option<f64>) -> Vec<String> {
    let Some(run) = &item.run else {
        return vec!["—".to_string()];
    };
    let mut lines = Vec::new();
    match data.frame_a {
        Some(frame) => {
            lines.push(format!("{}x{frame:.0} A AM", run.poles));
            lines.push(format!("{}x{:.0} A AD", run.poles, run.breaker_a));
        }
        None => lines.push(format!("{}x{:.0} A", run.poles, run.breaker_a)),
    }
    if !data.model.trim().is_empty() {
        lines.push(data.model.trim().to_uppercase());
    }
    let ka = data.interrupting_ka.or(board_ka);
    match (ka, data.brand.trim()) {
        (Some(ka), "") => lines.push(format!("{ka} kA")),
        (Some(ka), brand) => lines.push(format!("{ka} kA · {}", brand.to_uppercase())),
        (None, "") => {}
        (None, brand) => lines.push(brand.to_uppercase()),
    }
    if !data.trip_range.trim().is_empty() {
        lines.push(format!("AJUSTE {}", data.trip_range.trim()));
    }
    lines
}

/// Encabezado del tablero: nombre, carga instalada y demandada, factor de
/// demanda y sistema.
fn panel_header(project: &Project, panel: usize, result: &PanelResult) -> Vec<(String, Font, Ink)> {
    let p = &project.panels[panel];
    let system = bus_system(project, panel);
    // Sin fases asignadas no hay reparto por fase: se usa el factor de
    // potencia del tablero.
    let by_phase: f64 = result.phase_va.iter().sum();
    let installed_va = if by_phase > 0.0 {
        by_phase
    } else if result.power_factor > 0.0 {
        result.connected_w / result.power_factor
    } else {
        result.connected_w
    };
    vec![
        (
            format!("{}  «{}»", p.id, p.name.to_uppercase()),
            Font::Bold,
            Ink::Accent,
        ),
        (
            format!(
                "{:.3} kVA ({:.3} kW) INSTALADOS",
                installed_va / 1000.0,
                result.connected_w / 1000.0
            ),
            Font::Regular,
            Ink::Text,
        ),
        (
            format!(
                "{:.3} kVA ({:.3} kW) DEMANDADOS, F.D. {:.2}",
                result.demanded_va / 1000.0,
                result.demanded_w / 1000.0,
                result.demand_factor
            ),
            Font::Regular,
            Ink::Text,
        ),
        (
            format!(
                "{}, {}{}",
                voltage_label(&system),
                kind_wires(p.kind, p.feeder.has_neutral),
                frequency(project)
            ),
            Font::Regular,
            Ink::Text,
        ),
    ]
    .into_iter()
    .chain(board_line(&p.board))
    .chain(result.fault.as_ref().map(|f| {
        (
            if f.incomplete.is_empty() {
                format!("Icc = {:.2} kA ({})", f.total_ka, origin_text(f))
            } else {
                format!(
                    "Icc = {:.2} kA ({}, INCOMPLETA: FALTAN MOTORES)",
                    f.total_ka,
                    origin_text(f)
                )
            },
            Font::Regular,
            Ink::Text,
        )
    }))
    .collect()
}

fn frequency(project: &Project) -> String {
    match project.source.frequency_hz {
        Some(hz) => format!(", {hz:.0} Hz"),
        None => String::new(),
    }
}

/// Alguna barra tiene aportación de motores.
fn has_motor_contribution(result: &ProjectResult) -> bool {
    result
        .fault
        .iter()
        .flat_map(|f| f.source.iter())
        .chain(result.panels.iter().filter_map(|p| p.fault.as_ref()))
        .any(|b| b.motor_ka > 0.0)
}

fn origin_text(fault: &ieestudio_project::fault::BusFault) -> &'static str {
    match (&fault.origin, &fault.line_to_neutral) {
        (FaultOrigin::Supplier, _) => "DATO DE LA SUMINISTRADORA",
        (FaultOrigin::SupplierPrimary, _) => "DATO EN MEDIA TENSIÓN Y TRANSFORMADOR",
        (_, Some(_)) => "BUS INFINITO, FASE A NEUTRO",
        (_, None) => "BUS INFINITO",
    }
}

/// `BARRAS 225 A · 10 kA · MOD. NQ304… · SQUARE D`, con lo que haya.
fn board_line(board: &ieestudio_project::Board) -> Option<(String, Font, Ink)> {
    let mut parts = Vec::new();
    if let Some(a) = board.bus_a {
        parts.push(format!("BARRAS {a:.0} A"));
    }
    if let Some(ka) = board.interrupting_ka {
        parts.push(format!("{ka} kA"));
    }
    if !board.model.trim().is_empty() {
        parts.push(format!("MOD. {}", board.model.trim().to_uppercase()));
    }
    if !board.brand.trim().is_empty() {
        parts.push(board.brand.trim().to_uppercase());
    }
    (!parts.is_empty()).then(|| (parts.join(" · "), Font::Regular, Ink::Text))
}

/// Conexión, montaje y marca de un transformador, si se capturaron.
fn transformer_extra(
    connection: Option<ieestudio_project::Connection>,
    mounting: Option<ieestudio_project::Mounting>,
    brand: &str,
) -> Vec<(String, Font, Ink)> {
    let mut parts = Vec::new();
    if let Some(c) = connection {
        parts.push(c.label().to_uppercase());
    }
    if let Some(m) = mounting {
        parts.push(m.label().to_uppercase());
    }
    if !brand.trim().is_empty() {
        parts.push(format!("MARCA {}", brand.trim().to_uppercase()));
    }
    if parts.is_empty() {
        vec![]
    } else {
        vec![(parts.join(" · "), Font::Regular, Ink::Text)]
    }
}

/// Tensión y fases-hilos de una conexión en un sistema: `127 V, 1F-2H`.
fn connection_label(kind: CircuitKind, neutral: bool, system: &System) -> String {
    match kind.reference_voltage(system) {
        Ok(v) => format!("{v:.0} V, {}", kind_wires(kind, neutral)),
        Err(_) => kind_wires(kind, neutral).to_string(),
    }
}

struct Layout<'a> {
    project: &'a Project,
    result: &'a ProjectResult,
}

impl Layout<'_> {
    fn panel_result(&self, panel: usize) -> Option<&PanelResult> {
        let id = &self.project.panels[panel].id;
        self.result.panels.iter().find(|p| &p.id == id)
    }

    /// Dibuja un tablero desde `top`, con su alimentador bajando por
    /// `feed_x`. Si `fed` es verdadero, el interruptor y los conductores del
    /// alimentador ya se dibujaron en la columna del tablero padre. Devuelve
    /// dónde termina el tablero con todos sus subtableros.
    fn panel(&self, pt: &mut Painter, panel: usize, x0: f64, top: f64, fed: bool) -> f64 {
        let project = self.project;
        let p = &project.panels[panel];
        let Some(result) = self.panel_result(panel) else {
            return top;
        };
        let feed_x = x0 + PAD + COLUMN / 2.0;
        let mut y = top;
        let mut block_top = None;

        // Alimentador: interruptor y conductores.
        if !fed {
            pt.line((feed_x, y), (feed_x, y + 18.0), 1.3);
            pt.breaker(feed_x, y + 18.0);
            for (k, text) in breaker_lines(&result.feeder, &p.feeder.overrides.breaker, None)
                .into_iter()
                .enumerate()
            {
                pt.text(
                    (feed_x - 12.0, y + 26.0 + k as f64 * LINE),
                    text,
                    Font::Mono,
                    Ink::Text,
                    Anchor::End,
                );
            }
            let ctx = RunContext {
                label: format!("ALIM. {}", p.id),
                item: &result.feeder,
                overrides: &p.feeder.overrides,
            };
            let end = pt.lines((feed_x + 12.0, y + 50.0), conductor_lines(project, &ctx));
            block_top = Some(y + 50.0);
            y = end + 10.0;
        }

        // Transformador y su secundario (requisito I.2).
        if let Some(t) = &p.transformer {
            let tc = y + 22.0;
            pt.line((feed_x, y), (feed_x, tc - 15.0), 1.3);
            pt.symbol(Symbol::Transformer, (feed_x, tc));
            let primary = upstream_system(project, panel);
            pt.lines(
                (feed_x + 18.0, tc - 8.0),
                vec![
                    (
                        format!(
                            "TRANSFORMADOR {}",
                            known(t.kva, "kVA", "(POTENCIA SIN DATO)")
                        ),
                        Font::Bold,
                        Ink::Text,
                    ),
                    (
                        format!(
                            "{} – {}",
                            voltage_label(&primary),
                            voltage_label(&t.secondary)
                        ),
                        Font::Regular,
                        Ink::Text,
                    ),
                    (
                        format!("Z = {}", known(t.impedance_percent, "%", "SIN DATO")),
                        Font::Regular,
                        Ink::Text,
                    ),
                ]
                .into_iter()
                .chain(transformer_extra(t.connection, t.mounting, &t.brand))
                .collect(),
            );
            y = tc
                + 15.0
                + 12.0 * transformer_extra(t.connection, t.mounting, &t.brand).len() as f64;
            if let Some(secondary) = &result.secondary {
                pt.line((feed_x, y), (feed_x, y + 14.0), 1.3);
                y += 14.0;
                if t.secondary_protection {
                    pt.breaker(feed_x, y);
                    let lines = breaker_lines(
                        secondary,
                        &t.secondary_overrides.breaker,
                        p.board.interrupting_ka,
                    );
                    for (k, text) in lines.into_iter().enumerate() {
                        pt.text(
                            (feed_x - 12.0, y + 8.0 + k as f64 * LINE),
                            text,
                            Font::Mono,
                            Ink::Text,
                            Anchor::End,
                        );
                    }
                    y += 18.0;
                }
                let ctx = RunContext {
                    label: "SECUNDARIO".to_string(),
                    item: secondary,
                    overrides: &t.secondary_overrides,
                };
                let start = y + 12.0;
                let end = pt.lines((feed_x + 12.0, start), conductor_lines(project, &ctx));
                block_top = Some(start);
                y = end + 10.0;
            }
        }

        // Encabezado: junto al último bloque, o debajo si no hay.
        let header = panel_header(project, panel, result);
        let header_end = match block_top {
            Some(t) => pt.lines((feed_x + 210.0, t), header),
            None => {
                let end = pt.lines((feed_x + 12.0, y + 14.0), header);
                y = end;
                end
            }
        };
        y = y.max(header_end) + 12.0;

        // Barra, interruptores y lo que cuelga de cada uno, por filas.
        let slots = slots(project, panel);
        let (circuit_positions, subpanel_positions) = panel_positions(project, panel);
        let position_of = |slot: &Slot| -> Option<String> {
            match slot {
                Slot::Circuit(c) => circuit_positions.get(*c).cloned(),
                Slot::Subpanel(child) => children(project, panel)
                    .iter()
                    .position(|k| k == child)
                    .and_then(|k| subpanel_positions.get(k).cloned()),
            }
        };
        let rows: Vec<&[Slot]> = if slots.is_empty() {
            vec![&[]]
        } else {
            slots.chunks(MAX_COLUMNS).collect()
        };
        let width_of = |n: usize| (2.0 * PAD + n.max(1) as f64 * COLUMN).max(MIN_PANEL_WIDTH);
        let mut bus_y = y + 18.0;
        pt.line((feed_x, y - 12.0), (feed_x, bus_y), 1.3);
        pt.hotspots.push(Hotspot {
            min: (x0, top),
            size: (width_of(rows[0].len()), bus_y - top + 34.0),
            panel,
            circuit: None,
        });
        let mut previous_bus: Option<f64> = None;
        let mut pending = Vec::new();
        let mut bottom = bus_y;
        for (r, row) in rows.iter().enumerate() {
            let width = width_of(row.len());
            if let Some(prev) = previous_bus {
                // La barra continúa en la fila siguiente.
                pt.line((x0 + PAD / 2.0, prev), (x0 + PAD / 2.0, bus_y), 3.0);
            }
            pt.line(
                (x0 + PAD / 2.0, bus_y),
                (x0 + width - PAD / 2.0, bus_y),
                3.5,
            );
            // Renglones de los interruptores de esta fila.
            let breaker_rows: Vec<Vec<String>> = row
                .iter()
                .map(|slot| {
                    let item = match slot {
                        Slot::Circuit(c) => result.circuits.get(*c),
                        Slot::Subpanel(child) => self.panel_result(*child).map(|r| &r.feeder),
                    };
                    let data = match slot {
                        Slot::Circuit(c) => &p.circuits[*c].overrides.breaker,
                        Slot::Subpanel(child) => &project.panels[*child].feeder.overrides.breaker,
                    };
                    let mut lines =
                        item.map_or(vec![], |i| breaker_lines(i, data, p.board.interrupting_ka));
                    if let Slot::Circuit(c) = slot {
                        let phases: Vec<String> = p.circuits[*c]
                            .phases
                            .iter()
                            .map(|ph| format!("{ph:?}"))
                            .collect();
                        if !phases.is_empty() {
                            lines.push(format!(
                                "FASE{} {}",
                                if phases.len() > 1 { "S" } else { "" },
                                phases.join(",")
                            ));
                        }
                    }
                    lines
                })
                .collect();
            let tallest = breaker_rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
            // Renglones del bloque de conductores más alto, para alinear los
            // símbolos de toda la fila.
            let block_lines = row
                .iter()
                .map(|slot| match slot {
                    Slot::Circuit(c) => match (p.circuits.get(*c), result.circuits.get(*c)) {
                        (Some(circuit), Some(item)) => conductor_lines(
                            project,
                            &RunContext {
                                label: String::new(),
                                item,
                                overrides: &circuit.overrides,
                            },
                        )
                        .len(),
                        _ => 1,
                    },
                    Slot::Subpanel(child) => self.panel_result(*child).map_or(1, |r| {
                        conductor_lines(
                            project,
                            &RunContext {
                                label: String::new(),
                                item: &r.feeder,
                                overrides: &project.panels[*child].feeder.overrides,
                            },
                        )
                        .len()
                    }),
                })
                .max()
                .unwrap_or(1);
            let enclosure_top = bus_y - 18.0;
            let enclosure_bottom = bus_y + 44.0 + tallest as f64 * LINE;
            pt.rect(
                (x0 + 2.0, enclosure_top),
                (width - 4.0, enclosure_bottom - enclosure_top),
                true,
            );
            let mut row_bottom = enclosure_bottom;
            for (j, slot) in row.iter().enumerate() {
                let x = x0 + PAD + (j as f64 + 0.5) * COLUMN;
                pt.line((x, bus_y), (x, bus_y + 14.0), 1.3);
                pt.breaker(x, bus_y + 14.0);
                // Posición en el tablero, en un recuadro junto al interruptor.
                if let Some(position) = position_of(slot) {
                    let w = 8.0 + 4.6 * position.chars().count() as f64;
                    pt.rect((x - 16.0 - w, bus_y + 12.0), (w, 13.0), false);
                    pt.text_sized(
                        (x - 16.0 - w / 2.0, bus_y + 21.5),
                        position,
                        7.5,
                        Font::Mono,
                        Ink::Text,
                        Anchor::Middle,
                    );
                }
                pt.lines(
                    (x + 7.0, bus_y + 44.0),
                    breaker_rows[j]
                        .iter()
                        .enumerate()
                        .map(|(k, t)| {
                            let ink = if k == 0 { Ink::Text } else { Ink::Muted };
                            (t.clone(), Font::Mono, ink)
                        })
                        .collect(),
                );
                // Conductores, debajo del recuadro del tablero.
                let (ctx, symbol, labels, ink) = match slot {
                    Slot::Circuit(c) => {
                        let circuit = &p.circuits[*c];
                        let Some(item) = result.circuits.get(*c) else {
                            continue;
                        };
                        let system = bus_system(project, panel);
                        let name = match pt.mode {
                            Mode::Plan => circuit.name.to_uppercase(),
                            Mode::Review => circuit.name.clone(),
                        };
                        (
                            RunContext {
                                label: circuit.id.clone(),
                                item,
                                overrides: &circuit.overrides,
                            },
                            Symbol::Load(circuit.load.kind),
                            vec![
                                circuit.id.clone(),
                                shorten(&name, 24),
                                format!(
                                    "{:.3} kW · {}",
                                    item.load_w / 1000.0,
                                    connection_label(circuit.kind, circuit.has_neutral, &system)
                                ),
                            ],
                            status_ink(item.status),
                        )
                    }
                    Slot::Subpanel(child) => {
                        let cp = &project.panels[*child];
                        let Some(cr) = self.panel_result(*child) else {
                            continue;
                        };
                        let system = bus_system(project, *child);
                        (
                            RunContext {
                                label: format!("ALIM. {}", cp.id),
                                item: &cr.feeder,
                                overrides: &cp.feeder.overrides,
                            },
                            Symbol::Panel,
                            vec![
                                cp.id.clone(),
                                shorten(&cp.name.to_uppercase(), 24),
                                format!(
                                    "{:.3} kW · {}",
                                    cr.demanded_w / 1000.0,
                                    connection_label(cp.kind, cp.feeder.has_neutral, &system)
                                ),
                            ],
                            Ink::Accent,
                        )
                    }
                };
                pt.lines(
                    (x + 7.0, enclosure_bottom + 16.0),
                    conductor_lines(project, &ctx),
                );
                let sy = enclosure_bottom + 16.0 + (block_lines as f64 - 1.0) * LINE + 26.0;
                let half = Painter::symbol_half(symbol);
                pt.line((x, bus_y + 32.0), (x, sy - half), 1.3);
                pt.symbol(symbol, (x, sy));
                // Los datos de un subtablero van a la derecha de su símbolo:
                // la línea sigue hacia abajo hasta su dibujo.
                let subpanel = matches!(slot, Slot::Subpanel(_));
                let (lx, mut ly, anchor) = if subpanel {
                    (x + 22.0, sy - 8.0, Anchor::Start)
                } else {
                    (x, sy + 26.0, Anchor::Middle)
                };
                for (k, text) in labels.into_iter().enumerate() {
                    let (font, ink) = match k {
                        0 => (Font::Bold, ink),
                        1 if subpanel => (Font::Regular, Ink::Accent),
                        1 => (Font::Regular, Ink::Text),
                        _ => (Font::Regular, Ink::Muted),
                    };
                    pt.text((lx, ly), text, font, ink, anchor);
                    ly += LINE;
                }
                let column_bottom = ly;
                row_bottom = row_bottom.max(column_bottom);
                match slot {
                    Slot::Circuit(c) => pt.hotspots.push(Hotspot {
                        min: (x - COLUMN / 2.0, bus_y + 6.0),
                        size: (COLUMN, column_bottom - bus_y - 6.0),
                        panel,
                        circuit: Some(*c),
                    }),
                    Slot::Subpanel(child) => {
                        pt.hotspots.push(Hotspot {
                            min: (x - COLUMN / 2.0, bus_y + 6.0),
                            size: (COLUMN, column_bottom - bus_y - 6.0),
                            panel: *child,
                            circuit: None,
                        });
                        pending.push((*child, x, sy + half, r));
                    }
                }
            }
            bottom = row_bottom;
            previous_bus = Some(bus_y);
            bus_y = row_bottom + 46.0;
        }

        // Subtableros debajo, el de más a la derecha primero, para que las
        // líneas que bajan no crucen a los demás.
        for (child, x, from, _) in pending.into_iter().rev() {
            let child_top = bottom + 36.0;
            // La línea sigue hasta el subtablero con el texto de sus cargas a
            // la izquierda libre.
            pt.line((x, from), (x, child_top), 1.3);
            let child_x0 = x - PAD - COLUMN / 2.0;
            bottom = self.panel(pt, child, child_x0, child_top, true);
        }
        bottom
    }
}

/// Diagrama unifilar del proyecto para revisar en la aplicación.
pub fn single_line(project: &Project, result: &ProjectResult) -> Diagram {
    single_line_with(project, result, Options::REVIEW)
}

/// Diagrama unifilar como plano: blanco y negro, con la hoja completa.
pub fn plan(project: &Project, result: &ProjectResult) -> Diagram {
    single_line_with(project, result, Options::PLAN)
}

pub fn single_line_with(project: &Project, result: &ProjectResult, options: Options) -> Diagram {
    let mut pt = Painter {
        elements: Vec::new(),
        hotspots: Vec::new(),
        right: 0.0,
        bottom: 0.0,
        mode: options.mode,
        used: Vec::new(),
    };
    let layout = Layout { project, result };
    let roots: Vec<usize> = (0..project.panels.len())
        .filter(|&i| project.panels[i].parent.is_none())
        .collect();

    // Acometida (requisito I.1) y medidor.
    let first_feed = MARGIN + PAD + COLUMN / 2.0;
    let system = &project.source.system;
    let top = MARGIN;
    pt.path(
        vec![
            (first_feed, top),
            (first_feed, top + 12.0),
            (first_feed - 7.0, top + 18.0),
            (first_feed + 7.0, top + 26.0),
            (first_feed, top + 32.0),
            (first_feed, top + 44.0),
        ],
        1.3,
        false,
    );
    pt.path(
        vec![
            (first_feed - 5.0, top + 38.0),
            (first_feed + 5.0, top + 38.0),
            (first_feed, top + 46.0),
            (first_feed - 5.0, top + 38.0),
        ],
        1.3,
        false,
    );
    let source = &project.source;
    let supplier = source.supplier.trim().to_uppercase();
    let mut source_lines = vec![(
        shorten(&source.name.to_uppercase(), 34),
        Font::Bold,
        Ink::Text,
    )];
    // Con subestación la acometida llega en media tensión.
    let service = match &source.substation {
        Some(sub) => match sub.primary_kv {
            Some(kv) => format!("ACOMETIDA EN MEDIA TENSIÓN, {kv} kV"),
            None => "ACOMETIDA EN MEDIA TENSIÓN".to_string(),
        },
        None => "ACOMETIDA EN BAJA TENSIÓN".to_string(),
    };
    source_lines.push((
        if supplier.is_empty() {
            service
        } else {
            format!("{service} · {supplier}")
        },
        Font::Regular,
        Ink::Text,
    ));
    if source.substation.is_none() {
        source_lines.push((
            format!(
                "{}, {}{}",
                voltage_label(system),
                system_wires(system),
                frequency(project)
            ),
            Font::Regular,
            Ink::Text,
        ));
    }
    if let Some(source) = result.fault.as_ref().and_then(|f| f.source.as_ref()) {
        if source.origin == FaultOrigin::Supplier {
            source_lines.push((
                format!("Icc = {} kA (DATO DE LA SUMINISTRADORA)", source.base_ka),
                Font::Regular,
                Ink::Text,
            ));
        }
    }
    // El dato de media tensión se anota en la acometida, a su tensión.
    if let (Some(ka), Some(ieestudio_project::FaultLocation::Primary)) = (
        project.source.fault.available_ka,
        project.source.fault.location,
    ) {
        if result
            .fault
            .as_ref()
            .and_then(|f| f.source.as_ref())
            .is_some()
        {
            let kv = project
                .source
                .substation
                .as_ref()
                .and_then(|s| s.primary_kv)
                .map_or(String::new(), |kv| format!(" EN {kv} kV"));
            source_lines.push((
                format!("Icc = {ka} kA{kv} (DATO DE LA SUMINISTRADORA)"),
                Font::Regular,
                Ink::Text,
            ));
        }
    }
    pt.lines((first_feed + 26.0, top + 12.0), source_lines);
    let mut y = top + 46.0;

    // Subestación (requisito I.2): el transformador que alimenta la instalación.
    if let Some(sub) = &source.substation {
        let tc = y + 26.0;
        pt.line((first_feed, y), (first_feed, tc - 15.0), 1.3);
        pt.symbol(Symbol::Transformer, (first_feed, tc));
        let mut lines = vec![
            (
                format!(
                    "TRANSFORMADOR DE {}, Z = {}",
                    known(sub.kva, "kVA", "(POTENCIA SIN DATO)"),
                    if sub.impedance_from_table {
                        "MÍNIMA DE NORMA".to_string()
                    } else {
                        known(sub.impedance_percent, "%", "SIN DATO")
                    }
                ),
                Font::Bold,
                Ink::Text,
            ),
            (
                format!(
                    "{}{}, {}{}",
                    sub.primary_kv
                        .map_or(String::new(), |kv| format!("{kv} kV – ")),
                    voltage_label(system),
                    system_wires(system),
                    frequency(project)
                ),
                Font::Regular,
                Ink::Text,
            ),
        ];
        lines.extend(transformer_extra(sub.connection, sub.mounting, &sub.brand));
        if let Some(source) = result.fault.as_ref().and_then(|f| f.source.as_ref()) {
            if matches!(
                source.origin,
                FaultOrigin::Substation | FaultOrigin::SupplierPrimary
            ) {
                lines.push((
                    format!("Icc = {:.2} kA ({})", source.base_ka, origin_text(source)),
                    Font::Regular,
                    Ink::Text,
                ));
            }
        }
        let end = pt.lines((first_feed + 26.0, tc - 8.0), lines);
        y = (tc + 15.0).max(end + 4.0);
    }

    // Medidor, si lo hay.
    let mut meter_y = y;
    if source.meter {
        meter_y = y + 18.0;
        pt.line((first_feed, y), (first_feed, meter_y - 10.0), 1.3);
        pt.symbol(Symbol::Meter, (first_feed, meter_y));
        pt.text(
            (first_feed + 18.0, meter_y + 3.0),
            "MEDIDOR".to_string(),
            Font::Regular,
            Ink::Muted,
            Anchor::Start,
        );
    } else {
        meter_y -= 10.0;
    }
    let source_bottom = meter_y + 10.0;
    let panels_top = source_bottom + 18.0;
    pt.line((first_feed, source_bottom), (first_feed, panels_top), 1.3);

    // Tableros principales lado a lado; con más de uno, una barra los une.
    let mut x = MARGIN;
    let mut feeds = Vec::new();
    for &i in &roots {
        feeds.push(x + PAD + COLUMN / 2.0);
        x += panel_width(project, i) + 30.0;
    }
    if feeds.len() > 1 {
        let last = *feeds.last().unwrap_or(&first_feed);
        pt.line((first_feed, panels_top), (last, panels_top), 2.0);
    }
    let mut x = MARGIN;
    let mut bottom = panels_top;
    for &i in &roots {
        let end = layout.panel(&mut pt, i, x, panels_top, false);
        bottom = bottom.max(end);
        x += panel_width(project, i) + 30.0;
    }

    // Notas de las abreviaturas, siempre al pie.
    let mut y = bottom + 24.0;
    let mut notes: Vec<String> = vec![
        "F: conductor de fase · N: conductor puesto a tierra (neutro) · CPT: conductor de puesta a tierra de equipos".into(),
        "In: corriente nominal · L: longitud · %e: caída de tensión del tramo · Σ: caída acumulada desde la acometida".into(),
        "Calibres en AWG o kcmil; tubo en designación métrica y, entre paréntesis, en pulgadas · F.D.: factor de demanda".into(),
        "Junto a cada interruptor, su posición en el tablero".into(),
    ];
    let any_frame = project.panels.iter().any(|p| {
        p.feeder.overrides.breaker.frame_a.is_some()
            || p.circuits
                .iter()
                .any(|c| c.overrides.breaker.frame_a.is_some())
    });
    if any_frame {
        notes.push("AM: amperes de marco · AD: amperes de disparo".into());
    }
    match &result.fault {
        Some(study) => {
            let any_infinite = study
                .groups
                .iter()
                .any(|g| g.fault_ka.is_some() && g.infinite_bus);
            let with_origin = |o: FaultOrigin| {
                study
                    .groups
                    .iter()
                    .any(|g| g.fault_ka.is_some() && g.origin.as_ref() == Some(&o))
            };
            let any_primary = with_origin(FaultOrigin::SupplierPrimary);
            let mut parts = Vec::new();
            if any_infinite {
                parts.push(format!(
                    "la de BUS INFINITO toma la red sin impedancia, con la impedancia de placa del transformador reducida {:.0} %",
                    study.impedance_tolerance * 100.0
                ));
            }
            if any_primary {
                parts.push("la de DATO EN MEDIA TENSIÓN Y TRANSFORMADOR combina en cuadratura la impedancia de la red con la del transformador".to_string());
            }
            if (any_infinite || any_primary) && with_origin(FaultOrigin::Supplier) {
                parts.push("la de DATO DE LA SUMINISTRADORA se usa tal cual".to_string());
            }
            let mut note = if parts.is_empty() {
                "Icc: corriente de falla con el dato de la suministradora, sin descontar los alimentadores".to_string()
            } else {
                format!("Icc: corriente de falla estimada; {}", parts.join("; "))
            };
            if study.groups.iter().any(|g| g.fault_ka.is_some()) && has_motor_contribution(result) {
                let _ = write!(
                    note,
                    "; incluye {} veces la corriente de los motores de cada tensión y la que llega de otra tensión a través de su transformador",
                    study.multiplier
                );
            }
            if study
                .groups
                .iter()
                .any(|g| g.fault_ka.is_some() && g.line_to_neutral)
            {
                let _ = write!(
                    note,
                    "; en un secundario 1F-3H la de FASE A NEUTRO es {} veces la de fase a fase (método punto a punto de Eaton Bussmann)",
                    study.line_to_neutral_factor
                );
            }
            note.push_str("; es una cota superior. kA: capacidad interruptiva de catálogo");
            notes.push(note);
            if !study.incomplete_motors.is_empty() {
                notes.push(format!(
                    "La falla no incluye {} sin datos: la capacidad interruptiva que dependa de ellos no es concluyente",
                    if study.incomplete_motors.len() == 1 {
                        "1 motor".to_string()
                    } else {
                        format!("{} motores", study.incomplete_motors.len())
                    }
                ));
            }
            if !study.out_of_scope.is_empty() {
                notes.push(format!(
                    "Capacidad interruptiva no verificada en {}: falta la corriente de falla de la suministradora o los datos del transformador de la acometida",
                    study.out_of_scope.join(", ")
                ));
            }
        }
        None => notes.push(
            "Capacidad interruptiva no verificada: falta la corriente de falla de la suministradora o los datos del transformador de la acometida".into(),
        ),
    }
    if !options.sheet {
        for note in &notes {
            pt.text(
                (MARGIN, y),
                note.clone(),
                Font::Regular,
                Ink::Muted,
                Anchor::Start,
            );
            y += LINE;
        }
    }
    let content_right = pt.right.max(MARGIN + 600.0);
    let content_bottom = pt.bottom;

    if !options.sheet {
        return Diagram {
            width: content_right + MARGIN,
            height: content_bottom + MARGIN,
            elements: pt.elements,
            hotspots: pt.hotspots,
        };
    }
    sheet(pt, project, content_right, content_bottom, &notes)
}

/// La hoja: simbología, notas y cajetín a la derecha, y el marco.
fn sheet(
    mut pt: Painter,
    project: &Project,
    content_right: f64,
    content_bottom: f64,
    notes: &[String],
) -> Diagram {
    let sx = content_right + 36.0;
    let mut y = MARGIN;

    // Simbología: solo los símbolos que aparecen en el diagrama.
    let used = pt.used.clone();
    let row = 30.0;
    let table_height = 26.0 + used.len() as f64 * row;
    pt.rect((sx, y), (SIDE, table_height), false);
    pt.text_sized(
        (sx + 10.0, y + 17.0),
        "SIMBOLOGÍA (NMX-J-136-ANCE)".to_string(),
        10.0,
        Font::Bold,
        Ink::Text,
        Anchor::Start,
    );
    pt.line((sx, y + 26.0), (sx + SIDE, y + 26.0), 0.8);
    pt.line((sx + 60.0, y + 26.0), (sx + 60.0, y + table_height), 0.6);
    for (k, symbol) in used.iter().enumerate() {
        let ry = y + 26.0 + k as f64 * row;
        if k > 0 {
            pt.line((sx, ry), (sx + SIDE, ry), 0.5);
        }
        pt.symbol(*symbol, (sx + 30.0, ry + row / 2.0));
        let label = match symbol {
            Symbol::Breaker => "INTERRUPTOR TERMOMAGNÉTICO (AUTOMÁTICO)",
            Symbol::Meter => "EQUIPO DE MEDICIÓN",
            Symbol::Transformer => "TRANSFORMADOR",
            Symbol::Panel => "TABLERO DE DISTRIBUCIÓN",
            Symbol::Load(LoadKind::Lighting) => "SALIDA DE ALUMBRADO",
            Symbol::Load(LoadKind::Receptacles) => "RECEPTÁCULO (CONTACTOS)",
            Symbol::Load(LoadKind::Motor) => "MOTOR ELÉCTRICO",
            Symbol::Load(LoadKind::AirConditioning) => "SALIDA ESPECIAL: AIRE ACONDICIONADO",
            Symbol::Load(LoadKind::Appliance) => "SALIDA ESPECIAL: APARATO FIJO",
            Symbol::Load(LoadKind::KitchenEquipment) => "SALIDA ESPECIAL: EQUIPO DE COCINA",
            Symbol::Load(LoadKind::General) => "CARGA (SIN SÍMBOLO EN LA NORMA)",
        };
        pt.text(
            (sx + 70.0, ry + row / 2.0 + 3.0),
            label.to_string(),
            Font::Regular,
            Ink::Text,
            Anchor::Start,
        );
    }
    // El recuadro punteado del tablero.
    y += table_height + 10.0;
    pt.rect((sx + 12.0, y), (36.0, 16.0), true);
    pt.text(
        (sx + 70.0, y + 11.0),
        "LÍMITE DEL TABLERO".to_string(),
        Font::Regular,
        Ink::Text,
        Anchor::Start,
    );
    y += 40.0;

    // Notas.
    pt.text_sized(
        (sx, y),
        "NOTAS".to_string(),
        10.0,
        Font::Bold,
        Ink::Text,
        Anchor::Start,
    );
    y += 4.0;
    for (k, note) in notes.iter().enumerate() {
        for (j, part) in wrap(note, 58).into_iter().enumerate() {
            y += LINE;
            let text = if j == 0 {
                format!("{}. {part}", k + 1)
            } else {
                format!("   {part}")
            };
            pt.text((sx, y), text, Font::Regular, Ink::Text, Anchor::Start);
        }
    }
    let notes_bottom = y + 20.0;

    // Cajetín al pie de la hoja.
    let info = &project.info;
    let cells = [
        ("PROYECTO", project.name.clone()),
        ("CLIENTE", info.client.clone()),
        ("UBICACIÓN", info.location.clone()),
        ("ELABORÓ", info.prepared_by.clone()),
        ("RESPONSABLE", info.responsible.clone()),
        ("CÉDULA PROFESIONAL", info.license.clone()),
        ("REVISIÓN · FECHA", {
            let parts: Vec<&str> = [info.revision.as_str(), info.date.as_str()]
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .collect();
            parts.join(" · ")
        }),
    ];
    let cell = 20.0;
    let block_height = 30.0 + cells.len() as f64 * cell;
    let sheet_bottom = (content_bottom + 10.0).max(notes_bottom + block_height);
    let by = sheet_bottom - block_height;
    pt.rect((sx, by), (SIDE, block_height), false);
    pt.text_sized(
        (sx + 10.0, by + 20.0),
        "DIAGRAMA UNIFILAR".to_string(),
        12.0,
        Font::Bold,
        Ink::Text,
        Anchor::Start,
    );
    pt.line((sx, by + 30.0), (sx + SIDE, by + 30.0), 0.8);
    pt.line(
        (sx + 120.0, by + 30.0),
        (sx + 120.0, by + block_height),
        0.5,
    );
    for (k, (label, value)) in cells.iter().enumerate() {
        let cy = by + 30.0 + k as f64 * cell;
        if k > 0 {
            pt.line((sx, cy), (sx + SIDE, cy), 0.5);
        }
        pt.text_sized(
            (sx + 8.0, cy + 13.5),
            label.to_string(),
            7.5,
            Font::Regular,
            Ink::Muted,
            Anchor::Start,
        );
        let value = if value.trim().is_empty() {
            "—".to_string()
        } else {
            shorten(value, 36)
        };
        pt.text(
            (sx + 128.0, cy + 14.0),
            value,
            Font::Regular,
            Ink::Text,
            Anchor::Start,
        );
    }

    // Marco de la hoja.
    let width = sx + SIDE + MARGIN;
    let height = sheet_bottom + MARGIN;
    pt.rect((8.0, 8.0), (width - 16.0, height - 16.0), false);
    Diagram {
        width,
        height,
        elements: pt.elements,
        hotspots: pt.hotspots,
    }
}

/// Parte un texto en renglones de hasta `max` caracteres, por palabras.
fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > max {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
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
            Ink::Accent => "#b3261e",
            Ink::Complies => "#1f8a5b",
            Ink::Fails => "#b8401a",
            Ink::NotEvaluated => "#7a5a00",
        };
        let dash = |dashed: bool| {
            if dashed {
                r#" stroke-dasharray="6 4""#
            } else {
                ""
            }
        };
        let mut out = String::new();
        let _ = writeln!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}">"#,
            w = self.width,
            h = self.height
        );
        let _ = writeln!(out, r#"<rect width="100%" height="100%" fill="white"/>"#);
        for element in &self.elements {
            match element {
                Element::Path {
                    points,
                    width,
                    ink,
                    dashed,
                } => {
                    let pts: Vec<String> = points
                        .iter()
                        .map(|(x, y)| format!("{x:.1},{y:.1}"))
                        .collect();
                    let _ = writeln!(
                        out,
                        r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{width}" stroke-linejoin="round" stroke-linecap="round"{}/>"#,
                        pts.join(" "),
                        color(*ink),
                        dash(*dashed)
                    );
                }
                Element::Rect {
                    min,
                    size,
                    ink,
                    dashed,
                } => {
                    let _ = writeln!(
                        out,
                        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="none" stroke="{}" stroke-width="1.2"{}/>"#,
                        min.0,
                        min.1,
                        size.0,
                        size.1,
                        color(*ink),
                        dash(*dashed)
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
                        Font::Regular => "IBM Plex Sans, Arial, sans-serif",
                        Font::Bold => "IBM Plex Sans SmBld, Arial, sans-serif",
                        Font::Mono => "IBM Plex Mono, Consolas, monospace",
                    };
                    let weight = match font {
                        Font::Bold => r#" font-weight="600""#,
                        _ => "",
                    };
                    let anchor = match anchor {
                        Anchor::Start => "start",
                        Anchor::Middle => "middle",
                        Anchor::End => "end",
                    };
                    let _ = writeln!(
                        out,
                        r#"<text x="{:.1}" y="{:.1}" font-family="{family}" font-size="{size}"{weight} fill="{}" text-anchor="{anchor}">{}</text>"#,
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

    /// El diagrama en DXF (formato R12, el que abren todos los programas de
    /// CAD): líneas y textos en capas, una unidad por punto del diagrama,
    /// pensada como milímetros. Los textos van en Windows-1252; un carácter
    /// fuera de esa página va como `\U+XXXX`.
    pub fn to_dxf(&self) -> Vec<u8> {
        let mut out = String::new();
        let h = self.height;
        let pair = |out: &mut String, code: u32, value: &str| {
            let _ = writeln!(out, "{code:>3}\n{value}");
        };
        let layer = |ink: Ink, width: f64| -> (&'static str, i32) {
            match ink {
                Ink::Complies => ("VF-CUMPLE", 3),
                Ink::Fails => ("VF-NO-CUMPLE", 1),
                Ink::NotEvaluated => ("VF-SIN-EVALUAR", 40),
                Ink::Accent => ("VF-TABLEROS", 30),
                Ink::Muted => ("VF-NOTAS", 8),
                Ink::Text if width >= 3.0 => ("VF-BARRAS", 7),
                Ink::Text => ("VF-TRAZOS", 7),
            }
        };
        let layers = [
            ("VF-TRAZOS", 7),
            ("VF-BARRAS", 7),
            ("VF-TABLEROS", 30),
            ("VF-NOTAS", 8),
            ("VF-CUMPLE", 3),
            ("VF-NO-CUMPLE", 1),
            ("VF-SIN-EVALUAR", 40),
        ];
        // Encabezado: versión, página de códigos y extensión del dibujo.
        for (code, value) in [
            (0, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, "AC1009"),
        ] {
            pair(&mut out, code, value);
        }
        pair(&mut out, 9, "$DWGCODEPAGE");
        pair(&mut out, 3, "ANSI_1252");
        pair(&mut out, 9, "$EXTMIN");
        pair(&mut out, 10, "0");
        pair(&mut out, 20, "0");
        pair(&mut out, 9, "$EXTMAX");
        pair(&mut out, 10, &format!("{:.3}", self.width));
        pair(&mut out, 20, &format!("{h:.3}"));
        pair(&mut out, 0, "ENDSEC");
        // Tipos de línea: continua y discontinua (límite de los tableros).
        for (code, value) in [(0, "SECTION"), (2, "TABLES"), (0, "TABLE"), (2, "LTYPE")] {
            pair(&mut out, code, value);
        }
        pair(&mut out, 70, "2");
        for (name, description, pattern) in [
            ("CONTINUOUS", "Solid line", &[][..]),
            ("DASHED", "__ __ __", &[6.0, -4.0][..]),
        ] {
            pair(&mut out, 0, "LTYPE");
            pair(&mut out, 2, name);
            pair(&mut out, 70, "0");
            pair(&mut out, 3, description);
            pair(&mut out, 72, "65");
            pair(&mut out, 73, &pattern.len().to_string());
            let total: f64 = pattern.iter().map(|x: &f64| x.abs()).sum();
            pair(&mut out, 40, &format!("{total:.3}"));
            for element in pattern {
                pair(&mut out, 49, &format!("{element:.3}"));
            }
        }
        pair(&mut out, 0, "ENDTAB");
        // Capas.
        for (code, value) in [(0, "TABLE"), (2, "LAYER")] {
            pair(&mut out, code, value);
        }
        pair(&mut out, 70, &layers.len().to_string());
        for (name, color) in layers {
            pair(&mut out, 0, "LAYER");
            pair(&mut out, 2, name);
            pair(&mut out, 70, "0");
            pair(&mut out, 62, &color.to_string());
            pair(&mut out, 6, "CONTINUOUS");
        }
        pair(&mut out, 0, "ENDTAB");
        pair(&mut out, 0, "ENDSEC");

        pair(&mut out, 0, "SECTION");
        pair(&mut out, 2, "ENTITIES");
        let line = |out: &mut String, layer: &str, dashed: bool, a: (f64, f64), b: (f64, f64)| {
            pair(out, 0, "LINE");
            pair(out, 8, layer);
            if dashed {
                pair(out, 6, "DASHED");
            }
            pair(out, 10, &format!("{:.3}", a.0));
            pair(out, 20, &format!("{:.3}", h - a.1));
            pair(out, 11, &format!("{:.3}", b.0));
            pair(out, 21, &format!("{:.3}", h - b.1));
        };
        for element in &self.elements {
            match element {
                Element::Path {
                    points,
                    width,
                    ink,
                    dashed,
                } => {
                    let (name, _) = layer(*ink, *width);
                    for w in points.windows(2) {
                        line(&mut out, name, *dashed, w[0], w[1]);
                    }
                }
                Element::Rect {
                    min,
                    size,
                    ink,
                    dashed,
                } => {
                    let (name, _) = layer(*ink, 1.0);
                    let (x0, y0) = *min;
                    let (x1, y1) = (x0 + size.0, y0 + size.1);
                    let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)];
                    for w in corners.windows(2) {
                        line(&mut out, name, *dashed, w[0], w[1]);
                    }
                }
                Element::Text {
                    pos,
                    text,
                    size,
                    ink,
                    anchor,
                    ..
                } => {
                    let (name, _) = layer(*ink, 1.0);
                    let x = format!("{:.3}", pos.0);
                    let y = format!("{:.3}", h - pos.1);
                    pair(&mut out, 0, "TEXT");
                    pair(&mut out, 8, name);
                    pair(&mut out, 10, &x);
                    pair(&mut out, 20, &y);
                    // La altura de las mayúsculas es cerca del 70 % del tamaño
                    // de la letra.
                    pair(&mut out, 40, &format!("{:.3}", size * 0.7));
                    pair(&mut out, 1, &dxf_text(text));
                    let justify = match anchor {
                        Anchor::Start => 0,
                        Anchor::Middle => 1,
                        Anchor::End => 2,
                    };
                    if justify != 0 {
                        pair(&mut out, 72, &justify.to_string());
                        pair(&mut out, 11, &x);
                        pair(&mut out, 21, &y);
                    }
                }
            }
        }
        pair(&mut out, 0, "ENDSEC");
        pair(&mut out, 0, "EOF");
        out.chars().map(|c| cp1252(c).unwrap_or(b'?')).collect()
    }
}

/// Caracteres de Windows-1252 entre 0x80 y 0x9F, que no coinciden con Unicode.
const CP1252_EXTRA: [(char, u8); 27] = [
    ('€', 0x80),
    ('‚', 0x82),
    ('ƒ', 0x83),
    ('„', 0x84),
    ('…', 0x85),
    ('†', 0x86),
    ('‡', 0x87),
    ('ˆ', 0x88),
    ('‰', 0x89),
    ('Š', 0x8A),
    ('‹', 0x8B),
    ('Œ', 0x8C),
    ('Ž', 0x8E),
    ('‘', 0x91),
    ('’', 0x92),
    ('“', 0x93),
    ('”', 0x94),
    ('•', 0x95),
    ('–', 0x96),
    ('—', 0x97),
    ('˜', 0x98),
    ('™', 0x99),
    ('š', 0x9A),
    ('›', 0x9B),
    ('œ', 0x9C),
    ('ž', 0x9E),
    ('Ÿ', 0x9F),
];

/// Byte de Windows-1252 de un carácter, si lo tiene.
fn cp1252(c: char) -> Option<u8> {
    let code = u32::from(c);
    if code < 0x80 || (0xA0..=0xFF).contains(&code) {
        return u8::try_from(code).ok();
    }
    CP1252_EXTRA.iter().find(|(x, _)| *x == c).map(|(_, b)| *b)
}

/// Texto para DXF: lo que no está en Windows-1252 va como `\U+XXXX`, que
/// entienden AutoCAD y los programas compatibles.
fn dxf_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if cp1252(c).is_some() && !c.is_control() {
                c.to_string()
            } else {
                format!("\\U+{:04X}", u32::from(c))
            }
        })
        .collect()
}
