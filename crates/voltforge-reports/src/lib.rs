//! Memoria de cálculo en PDF.
//!
//! El documento se escribe como texto Typst a partir del proyecto y de su
//! resultado, y Typst lo compone. Todo el contenido sale del resultado ya
//! calculado: aquí no se calcula nada.

use std::fmt::Write as _;

use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;
use voltforge_core::{CircuitKind, System};
use voltforge_diagrams::single_line;
use voltforge_project::quick::{QuickInput, QuickResult};
use voltforge_project::verdict::Verdict;
use voltforge_project::{
    insulation_name, material_name, size_label, PanelResult, Project, ProjectInfo, ProjectResult,
    RunResult, Status, Step,
};
use voltforge_rules::{cite_all, Profile, Rule};

static FONTS: [&[u8]; 3] = [
    include_bytes!("../../../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/IBMPlexMono-Regular.ttf"),
];

/// Estilo y funciones del documento.
const PREAMBLE: &str = r##"
#let tinta = rgb("#1b222c")
#let gris = rgb("#5d6877")
#let linea = rgb("#d5dae1")
#let acento = rgb("#b97a00")
#let bien = rgb("#1f8a5b")
#let mal = rgb("#b8401a")
#let duda = rgb("#7a5a00")

#set document(title: titulo, author: "VoltForge")
#set page(
  paper: "us-letter",
  margin: (x: 1.8cm, top: 2.2cm, bottom: 2cm),
  header: context {
    set text(8pt, fill: gris)
    grid(columns: (1fr, auto), titulo, [VoltForge · #perfil])
    v(-6pt)
    line(length: 100%, stroke: 0.5pt + linea)
  },
  footer: context {
    set text(8pt, fill: gris)
    grid(
      columns: (1fr, auto),
      [Generado con VoltForge #version. No constituye aprobación profesional.],
      [Página #counter(page).display() de #counter(page).final().first()],
    )
  },
)
#set text(font: "IBM Plex Sans", size: 9.5pt, fill: tinta, lang: "es")
#set par(leading: 0.55em)
#show heading.where(level: 1): it => block(above: 1.6em, below: 0.8em, text(13pt, font: "IBM Plex Sans SmBld", it.body))
#show heading.where(level: 2): it => block(above: 1.4em, below: 0.7em, text(10.5pt, font: "IBM Plex Sans SmBld", it.body))

#let mono(t) = text(font: "IBM Plex Mono", size: 8.5pt, t)
#let estado(e) = {
  let color = if e == "Cumple" { bien } else if e == "No cumple" { mal } else { duda }
  text(fill: color, font: "IBM Plex Sans SmBld", e)
}
#let rotulo(t) = text(7.5pt, fill: gris, tracking: 0.4pt, upper(t))

#let datos(..pares) = table(
  columns: (5.2cm, 1fr),
  stroke: (x, y) => (bottom: 0.5pt + linea),
  inset: (x: 0pt, y: 4.5pt),
  ..pares.pos().enumerate().map(((i, c)) => if calc.even(i) { text(fill: gris, c) } else { c })
)

#let paso(n, nombre, formula, sustitucion, nota, ref) = block(
  breakable: false,
  width: 100%,
  inset: (left: 0pt, y: 3pt),
  grid(
    columns: (0.7cm, 1fr),
    text(11pt, font: "IBM Plex Sans SmBld", fill: acento, str(n)),
    {
      text(font: "IBM Plex Sans SmBld", nombre)
      if formula != none { linebreak(); text(fill: gris, mono(formula)) }
      linebreak()
      mono(sustitucion)
      if nota != none { linebreak(); text(8.5pt, fill: gris, nota) }
      if ref != none { linebreak(); text(7.5pt, fill: acento, "Referencia: " + ref) }
    },
  ),
)

#let cuadro(..filas) = {
  set text(8pt)
  table(
    columns: (auto, 1fr, auto, auto, auto, auto, auto, auto, auto, auto, auto, auto),
    align: (x, y) => if x in (2, 3, 4, 9, 10) { right } else { left },
    stroke: (x, y) => (bottom: 0.5pt + linea),
    inset: (x: 4pt, y: 4.5pt),
    table.header(
      ..("ID", "Descripción", "Carga W", "I A", "L m", "Calibre", "Tierra", "Tubo", "ITM", "e %", "Σe %", "Estado").map(rotulo),
    ),
    ..filas.pos(),
  )
}
"##;

/// Texto como cadena de Typst, con comillas y diagonales invertidas escapadas.
fn q(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn opt(text: &Option<String>) -> String {
    text.as_deref().map_or("none".to_string(), q)
}

fn system_label(system: &System) -> String {
    match (system.line_to_line_v, system.line_to_neutral_v) {
        (Some(ll), Some(ln)) => format!("{ll:.0}/{ln:.0} V"),
        (Some(ll), None) => format!("{ll:.0} V, sin neutro"),
        (None, Some(ln)) => format!("{ln:.0} V, una fase y neutro"),
        (None, None) => "sin tensión definida".to_string(),
    }
}

fn kind_label(kind: CircuitKind) -> &'static str {
    match kind {
        CircuitKind::LineToNeutral => "1 fase y neutro",
        CircuitKind::LineToLine => "2 fases",
        CircuitKind::ThreePhase => "3 fases",
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "Sí"
    } else {
        "No"
    }
}

fn today() -> String {
    const MONTHS: [&str; 12] = [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ];
    use chrono::Datelike;
    let now = chrono::Local::now();
    format!(
        "{} de {} de {}",
        now.day(),
        MONTHS[now.month0() as usize],
        now.year()
    )
}

fn head(out: &mut String, title: &str, profile: &str) {
    let _ = writeln!(out, "#let titulo = {}", q(title));
    let _ = writeln!(out, "#let perfil = {}", q(&profile.to_uppercase()));
    let _ = writeln!(out, "#let version = {}", q(env!("CARGO_PKG_VERSION")));
    out.push_str(PREAMBLE);
}

fn data_table(out: &mut String, rows: &[(&str, String)]) {
    out.push_str("#datos(\n");
    for (name, value) in rows {
        let _ = writeln!(out, "  {}, {},", q(name), q(value));
    }
    out.push_str(")\n");
}

fn steps(out: &mut String, steps: &[Step]) {
    for (i, step) in steps.iter().enumerate() {
        let _ = writeln!(
            out,
            "#paso({}, {}, {}, {}, {}, {})",
            i + 1,
            q(&step.title),
            opt(&step.formula),
            q(&step.substitution),
            opt(&step.note),
            opt(&cite_all(&step.references))
        );
    }
}

/// Espacio para firmas. Con los datos de la obra, los nombres ya impresos.
fn signatures(out: &mut String, info: Option<&ProjectInfo>) {
    let name = |text: &str| {
        if text.trim().is_empty() {
            String::new()
        } else {
            format!("#text(9.5pt, {})\\ ", q(text.trim()))
        }
    };
    let (prepared, responsible) = match info {
        Some(i) => {
            let license = if i.license.trim().is_empty() {
                String::new()
            } else {
                format!("Cédula profesional {}", i.license.trim())
            };
            (
                name(&i.prepared_by),
                format!("{}{}", name(&i.responsible), name(&license)),
            )
        }
        None => (String::new(), String::new()),
    };
    let _ = writeln!(
        out,
        r#"
#v(1.6cm)
#block(breakable: false, grid(
  columns: (1fr, 1fr),
  column-gutter: 1.5cm,
  [#line(length: 100%, stroke: 0.5pt + tinta) {prepared}#text(8.5pt, fill: gris)[Elaboró: nombre, firma y fecha]],
  [#line(length: 100%, stroke: 0.5pt + tinta) {responsible}#text(8.5pt, fill: gris)[Revisó y aprobó: nombre, cédula profesional y firma]],
))"#
    );
}

/// Cálculo, verificación normativa y aprobación profesional, en tres filas.
fn verdict_table(out: &mut String, verdict: &Verdict, code: &str) {
    out.push_str("#table(columns: (4.2cm, 2.6cm, 1fr), stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 0pt, y: 5pt),\n");
    for (level, status, detail) in verdict.lines(code) {
        let state = match status {
            Status::Complies => "Cumple",
            Status::Fails => "No cumple",
            Status::NotEvaluated => "Pendiente",
        };
        let label = detail.split(':').next().unwrap_or_default().to_string();
        let explanation = detail
            .split_once(": ")
            .map_or(String::new(), |(_, e)| e.to_string());
        let _ = writeln!(
            out,
            "  text(fill: gris, {}), text(font: \"IBM Plex Sans SmBld\", fill: if {} == \"Cumple\" {{ bien }} else if {} == \"No cumple\" {{ mal }} else {{ duda }}, {}), {},",
            q(level),
            q(state),
            q(state),
            q(&label),
            q(&explanation)
        );
    }
    out.push_str(")\n");
}

/// Carga del alimentador por grupo, según el artículo 220.
fn demand_table(out: &mut String, panel: &voltforge_project::PanelResult) {
    if panel.demand.is_empty() {
        return;
    }
    out.push_str("#table(columns: (4.4cm, 2.1cm, 1fr, 2.1cm), stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 0pt, y: 4pt), align: (left, right, left, right),\n");
    out.push_str("  text(fill: gris, size: 8pt)[Carga del alimentador], text(fill: gris, size: 8pt)[Conectada VA], text(fill: gris, size: 8pt)[#h(8pt)Factor de demanda], text(fill: gris, size: 8pt)[Calculada VA],\n");
    for line in &panel.demand {
        let cite = cite_all(&line.references).map_or(String::new(), |c| format!(" · {c}"));
        let _ = writeln!(
            out,
            "  {}, {}, [#h(8pt)#text(size: 8pt, {})#text(size: 7pt, fill: gris, {})], {},",
            q(&line.label),
            q(&format!("{:.0}", line.connected_va)),
            q(&line.detail),
            q(&cite),
            q(&format!("{:.0}", line.demanded_va)),
        );
    }
    let _ = writeln!(
        out,
        "  text(font: \"IBM Plex Sans SmBld\", \"Total\"), {}, [], text(font: \"IBM Plex Sans SmBld\", {}),",
        q(&format!(
            "{:.0}",
            panel.demand.iter().map(|l| l.connected_va).sum::<f64>()
        )),
        q(&format!("{:.0}", panel.demanded_va)),
    );
    out.push_str(")\n");
}

fn schedule_row(out: &mut String, id: &str, item: &RunResult) {
    let cells: Vec<String> = match &item.run {
        Some(run) => {
            let size = if run.conductors_per_phase > 1 {
                format!("{}×{}", run.conductors_per_phase, run.size.0)
            } else {
                run.size.0.clone()
            };
            vec![
                format!("{:.0}", item.load_w),
                format!("{:.2}", run.nominal_a),
                format!("{:.1}", run.length_m),
                size,
                run.grounding_size.0.clone(),
                run.conduit
                    .as_ref()
                    .map_or("—".to_string(), |c| c.short_label()),
                format!("{}P-{:.0}", run.poles, run.breaker_a),
                format!("{:.2}", run.drop_percent),
                format!("{:.2}", run.cumulative_drop_percent),
            ]
        }
        None => {
            let mut cells = vec![format!("{:.0}", item.load_w)];
            cells.extend(std::iter::repeat_n("—".to_string(), 8));
            cells
        }
    };
    let _ = write!(out, "  {}, {}, ", q(id), q(&item.name));
    for cell in cells {
        let _ = write!(out, "mono({}), ", q(&cell));
    }
    let _ = writeln!(out, "estado({}),", q(item.status.label()));
}

fn panel_runs(panel: &PanelResult) -> impl Iterator<Item = (String, &RunResult)> {
    std::iter::once(("ALIM".to_string(), &panel.feeder))
        .chain(panel.secondary.iter().map(|s| ("SEC".to_string(), s)))
        .chain(panel.circuits.iter().map(|c| (c.id.clone(), c)))
}

/// Texto Typst de la memoria de cálculo de un proyecto.
pub fn project_source(project: &Project, result: &ProjectResult) -> String {
    let s = &project.settings;
    let profile = Profile::by_id(&project.profile).unwrap_or(Profile::nom_001_sede_2012());
    let mut out = String::new();
    let info = &project.info;
    let revision = info.revision.trim();
    head(
        &mut out,
        &if revision.is_empty() {
            format!("Memoria de cálculo · {}", project.name)
        } else {
            format!("Memoria de cálculo · {} · Rev. {revision}", project.name)
        },
        &project.profile,
    );
    let date = if info.date.trim().is_empty() {
        today()
    } else {
        info.date.trim().to_string()
    };

    let _ = writeln!(
        out,
        "#text(18pt, font: \"IBM Plex Sans SmBld\", \"Memoria de cálculo eléctrico\")\n\n#text(12pt, {})\n\n#text(fill: gris, {})",
        q(&project.name),
        q(&format!(
            "Instalación eléctrica de baja tensión calculada con la {} · {date}",
            project.profile.to_uppercase(),
        ))
    );

    let mut work: Vec<(&str, String)> = Vec::new();
    for (label, value) in [
        ("Cliente", &info.client),
        ("Ubicación", &info.location),
        ("Revisión", &info.revision),
        ("Elaboró", &info.prepared_by),
        ("Responsable", &info.responsible),
        ("Cédula profesional", &info.license),
    ] {
        if !value.trim().is_empty() {
            work.push((label, value.trim().to_string()));
        }
    }
    if !work.is_empty() {
        out.push_str("\n= Datos de la obra\n");
        data_table(&mut out, &work);
    }

    out.push_str("\n= Datos y criterios de diseño\n");
    data_table(
        &mut out,
        &[
            (
                "Suministro",
                format!(
                    "{}, {}",
                    project.source.name,
                    system_label(&project.source.system)
                ),
            ),
            (
                "Tipo de inmueble",
                project
                    .occupancy
                    .as_deref()
                    .and_then(|id| profile.occupancy(id))
                    .map_or("Sin elegir".to_string(), |o| {
                        format!("{} ({} VA/m², Tabla 220-12)", o.label, o.va_per_m2)
                    }),
            ),
            ("Cortocircuito de la fuente", {
                let motors = result
                    .motor_fault_ka
                    .filter(|m| *m > 0.0)
                    .map_or(String::new(), |m| {
                        format!(", más {:.2} kA de aportación de motores", m)
                    });
                match (result.source_fault_ka, &project.source.fault) {
                        (Some(ka), f) if f.available_ka.is_some() => format!(
                            "{ka:.2} kA: {} kA disponibles (dato de la compañía suministradora){motors}",
                            f.available_ka.unwrap_or_default()
                        ),
                        (Some(ka), f) => format!(
                            "{ka:.2} kA: transformador de {} kVA con {} % de impedancia{motors}",
                            f.transformer_kva.unwrap_or_default(),
                            f.transformer_impedance_percent.unwrap_or_default()
                        ),
                        (None, _) => {
                            "Sin datos: no se verificó la capacidad interruptiva".to_string()
                        }
                    }
            }),
            (
                "Conductor al electrodo de puesta a tierra",
                result.grounding_electrode.as_ref().map_or(
                    "Sin alimentador calculado desde la fuente".to_string(),
                    |g| {
                        format!(
                            "{} de cobre para una acometida de {} (Tabla 250-66)",
                            size_label(&g.size),
                            g.service
                        )
                    },
                ),
            ),
            ("Conductor", capitalize(material_name(s.material))),
            ("Aislamiento", insulation_name(s.insulation).to_string()),
            ("Canalización", s.conduit.to_string()),
            ("Temperatura ambiente", format!("{} °C", s.ambient_c)),
            ("Lugar mojado", yes_no(s.wet_location).to_string()),
            (
                "Conductor de puesta a tierra",
                if s.bare_grounding {
                    "Desnudo"
                } else {
                    "Aislado"
                }
                .to_string(),
            ),
            (
                "Caída de tensión permitida",
                format!(
                    "{} % en circuitos derivados, {} % en alimentadores y {} % acumulada",
                    s.branch_drop_limit_percent,
                    s.feeder_drop_limit_percent,
                    s.total_drop_limit_percent
                ),
            ),
            (
                "Resistencia para la caída de tensión",
                match s.resistance_temperature_c {
                    Some(t) => format!(
                        "Ajustada a {t} °C ({})",
                        profile.clause(Rule::ResistanceTemperature)
                    ),
                    None => format!(
                        "La de la tabla de impedancias, a 75 °C ({})",
                        profile.clause(Rule::Impedance)
                    ),
                },
            ),
            (
                "Interruptores",
                format!(
                    "Capacidades normalizadas ({}), {} 16, 32 y 63 A",
                    profile.clause(Rule::StandardRatings),
                    if s.include_iec_ratings {
                        "incluidas"
                    } else {
                        "sin"
                    }
                ),
            ),
            (
                "Norma",
                format!(
                    "{}, {} (edición {})",
                    profile.document.code, profile.document.title, profile.document.edition
                ),
            ),
        ],
    );

    out.push_str("\n= Resumen\n");
    verdict_table(&mut out, &result.verdict, &profile.document.code);
    let _ = writeln!(
        out,
        "De los tramos calculados, #text(fill: bien, font: \"IBM Plex Sans SmBld\")[{} cumplen], #text(fill: mal, font: \"IBM Plex Sans SmBld\")[{} no cumplen] y #text(fill: duda, font: \"IBM Plex Sans SmBld\")[{} no se evaluaron]. Un tramo «no evaluado» no está aprobado: falta un dato o su regla aún no está implementada.",
        result.complies, result.fails, result.not_evaluated
    );

    for panel in &result.panels {
        let _ = writeln!(out, "\n== Cuadro de cargas · #{};", q(&panel.name));
        let [a, b, c] = panel.phase_va;
        let mut fault = panel.fault_ka.map_or(String::new(), |ka| {
            format!(" Corriente de cortocircuito en barras: {ka:.2} kA.")
        });
        if let Some(t) = &panel.transformer {
            fault.push_str(&format!(
                " Se alimenta por un transformador de {} kVA (primario {}, secundario {}), cargado al {:.0} %.",
                t.kva,
                t.primary_a.map_or("—".into(), |a| format!("{a:.2} A")),
                t.secondary_a.map_or("—".into(), |a| format!("{a:.2} A")),
                t.load_percent
            ));
        }
        let balance = match panel.imbalance_percent {
            Some(i) => format!(
                " Carga por fase: A {a:.0} VA, B {b:.0} VA, C {c:.0} VA; desbalance de {i:.1} %."
            ),
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "#text(fill: gris, {})",
            q(&format!(
                "Carga conectada de {:.0} W y carga calculada del alimentador de {:.0} VA ({:.0} W, factor resultante {:.2}), factor de potencia {:.2}.{balance}{fault}",
                panel.connected_w, panel.demanded_va, panel.demanded_w, panel.demand_factor, panel.power_factor
            ))
        );
        demand_table(&mut out, panel);
        out.push_str("#cuadro(\n");
        for (id, item) in panel_runs(panel) {
            schedule_row(&mut out, &id, item);
        }
        out.push_str(")\n");
    }

    // Diagrama unifilar en una hoja horizontal, ajustado a la página.
    let svg = single_line(project, result).to_svg();
    let _ = writeln!(
        out,
        "\n#page(flipped: true)[\n= Diagrama unifilar\n#block(height: 1fr, width: 100%, image(bytes({}), format: \"svg\", width: 100%, height: 100%, fit: \"contain\"))\n]",
        q(&svg)
    );

    // Observaciones: todo lo que no cumple o no se evaluó, y los avisos.
    let mut notes = Vec::new();
    for panel in &result.panels {
        for (id, item) in panel_runs(panel) {
            for check in item.checks.iter().filter(|c| c.status != Status::Complies) {
                let who = if id == "ALIM" {
                    format!("Alimentador de {}", panel.name)
                } else if id == "SEC" {
                    format!("Secundario del transformador de {}", panel.name)
                } else {
                    format!("{id} {}", item.name)
                };
                let cite = cite_all(&check.references).map_or(String::new(), |c| format!(" ({c})"));
                notes.push(format!(
                    "- #estado({}) · #{}; · #{}; #{};",
                    q(check.status.label()),
                    q(&who),
                    q(&format!("{}:", check.name)),
                    q(&format!("{}{cite}", check.detail))
                ));
            }
        }
    }
    for warning in &result.warnings {
        notes.push(format!(
            "- #text(fill: duda, font: \"IBM Plex Sans SmBld\")[Aviso] · #{};",
            q(warning)
        ));
    }
    out.push_str("\n= Observaciones\n");
    if notes.is_empty() {
        out.push_str("Todos los tramos cumplen y no hay avisos.\n");
    } else {
        out.push_str(&notes.join("\n"));
        out.push('\n');
    }

    out.push_str("\n= Procedimiento de cálculo\n");
    out.push_str("#text(fill: gris)[Cada tramo muestra la fórmula, los valores sustituidos y la tabla o artículo de la norma. Los valores se presentan redondeados; el cálculo se hace sin redondear pasos intermedios.]\n");
    for panel in &result.panels {
        for (id, item) in panel_runs(panel) {
            let title = if id == "ALIM" {
                format!("Alimentador de {}", panel.name)
            } else if id == "SEC" {
                format!("Secundario del transformador de {}", panel.name)
            } else {
                format!("{id} · {}", item.name)
            };
            let _ = writeln!(
                out,
                "\n#block(breakable: false)[\n== #{};\n#estado({}) #h(6pt) #text(fill: gris, {})\n]",
                q(&title),
                q(item.status.label()),
                q(&format!("Carga de {:.0} W", item.load_w))
            );
            match &item.run {
                Some(run) => steps(&mut out, &run.steps),
                None => {
                    for check in &item.checks {
                        let _ = writeln!(
                            out,
                            "#{};",
                            q(&format!("{}: {}.", check.name, check.detail))
                        );
                    }
                }
            }
        }
    }

    signatures(&mut out, Some(&project.info));
    out
}

/// Texto Typst de la hoja de un cálculo rápido.
pub fn quick_source(input: &QuickInput, result: &QuickResult) -> String {
    let mut out = String::new();
    head(&mut out, "Hoja de cálculo · circuito", "nom-001-sede-2012");
    let _ = writeln!(
        out,
        "#text(18pt, font: \"IBM Plex Sans SmBld\", \"Hoja de cálculo de un circuito\")\n\n#text(fill: gris, {})",
        q(&format!(
            "Corriente, protección, conductor, tubería y caída de tensión con la NOM-001-SEDE-2012 · {}",
            today()
        ))
    );

    out.push_str("\n= Datos\n");
    let load = if input.load_in_amperes {
        format!("{} A, capturada como corriente", input.current_a)
    } else {
        format!("{} W", input.power_w)
    };
    let conductor = if input.conductors_per_phase > 1 {
        format!(
            "{} de {} por fase, {} {}",
            input.conductors_per_phase,
            size_label(&input.size),
            material_name(input.material),
            insulation_name(input.insulation)
        )
    } else {
        format!(
            "{} de {} {}",
            size_label(&input.size),
            material_name(input.material),
            insulation_name(input.insulation)
        )
    };
    data_table(
        &mut out,
        &[
            (
                "Sistema y conexión",
                format!(
                    "{}, {}",
                    system_label(&input.system),
                    kind_label(input.kind)
                ),
            ),
            ("Carga", load),
            ("Factor de potencia", format!("{:.2}", input.power_factor)),
            ("Carga continua", yes_no(input.continuous).to_string()),
            ("Longitud", format!("{} m", input.length_m)),
            ("Conductor", conductor),
            ("Canalización", input.conduit.to_string()),
            ("Temperatura ambiente", format!("{} °C", input.ambient_c)),
            (
                "Límite de caída de tensión",
                format!("{} %", input.limit_percent),
            ),
        ],
    );

    out.push_str("\n= Resultado\n");
    let mut rows = vec![("Corriente nominal", format!("{:.2} A", result.current_a))];
    match &result.protection {
        Ok(p) => {
            rows.push(("Corriente de diseño", format!("{:.2} A", p.design_a)));
            rows.push(("Interruptor", format!("{}P-{:.0} A", p.poles, p.breaker_a)));
            rows.push((
                "Conductor",
                format!(
                    "{}: {:.2} A permitidos, {}",
                    size_label(&input.size),
                    p.allowable_ampacity_a,
                    p.status.label().to_lowercase()
                ),
            ));
            if let Some(ground) = &p.grounding_size {
                rows.push(("Puesta a tierra", size_label(ground)));
            }
            if let Some(conduit) = &p.conduit {
                rows.push(("Tubería", conduit.clone()));
            }
        }
        Err(e) => {
            rows.push(("Protección y conductor", format!("No evaluado: {e}")));
        }
    }
    rows.push((
        "Caída de tensión",
        format!(
            "{:.2} V, {:.2} % de {:.0} V: {}",
            result.drop.drop_v,
            result.drop.drop_percent,
            result.drop.reference_voltage_v,
            result.status.label().to_lowercase()
        ),
    ));
    verdict_table(&mut out, &result.verdict(), "NOM-001-SEDE-2012");
    data_table(&mut out, &rows);
    for warning in &result.warnings {
        let _ = writeln!(
            out,
            "#text(fill: duda, font: \"IBM Plex Sans SmBld\")[Aviso] · #{};\n",
            q(warning)
        );
    }

    out.push_str("\n= Procedimiento\n");
    steps(&mut out, &result.steps);

    out.push_str("\n= Alternativas\n");
    for rec in &result.recommendations {
        let _ = writeln!(
            out,
            "- #text(font: \"IBM Plex Sans SmBld\", {}) · #mono({}) · #text(fill: gris, {})",
            q(&rec.title),
            q(&rec.value),
            q(&rec.detail)
        );
    }
    out.push_str("\n#text(8.5pt, fill: gris)[Supone lugar seco, carga lineal, tierra desnuda y, con conductores en paralelo, un juego por tubo.]\n");

    signatures(&mut out, None);
    out
}

fn compose(source: String) -> Result<PagedDocument, String> {
    let engine = TypstEngine::builder()
        .main_file(source)
        .fonts(FONTS)
        .build();
    engine
        .compile()
        .output
        .map_err(|e| format!("no se pudo componer el documento: {e}"))
}

fn render(source: String) -> Result<Vec<u8>, String> {
    let document = compose(source)?;
    typst_pdf::pdf(&document, &Default::default())
        .map_err(|e| format!("no se pudo generar el PDF: {e:?}"))
}

/// Página rasterizada para mostrarla en pantalla: RGBA de 8 bits.
pub struct PageImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

fn rasterize(source: String, pixels_per_point: f64) -> Result<Vec<PageImage>, String> {
    let document = compose(source)?;
    let options = typst_render::RenderOptions {
        pixel_per_pt: typst_utils::Scalar::new(pixels_per_point),
        ..Default::default()
    };
    Ok(document
        .pages()
        .iter()
        .map(|page| {
            let pixmap = typst_render::render(page, &options);
            // Las páginas son opacas: el RGBA premultiplicado es igual al directo.
            PageImage {
                width: pixmap.width(),
                height: pixmap.height(),
                rgba: pixmap.data().to_vec(),
            }
        })
        .collect())
}

/// Páginas de la memoria de un proyecto como imágenes, para la vista previa.
pub fn project_pages(
    project: &Project,
    result: &ProjectResult,
    pixels_per_point: f64,
) -> Result<Vec<PageImage>, String> {
    rasterize(project_source(project, result), pixels_per_point)
}

/// Memoria de cálculo de un proyecto, en PDF.
pub fn project_pdf(project: &Project, result: &ProjectResult) -> Result<Vec<u8>, String> {
    render(project_source(project, result))
}

/// Hoja de un cálculo rápido, en PDF.
pub fn quick_pdf(input: &QuickInput, result: &QuickResult) -> Result<Vec<u8>, String> {
    render(quick_source(input, result))
}
