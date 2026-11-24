//! Sistema visual de IEEStudio.
//!
//! Obsidiana, grafito y pizarra dominan las superficies; el color comunica
//! significado y no decora:
//!
//! - **Amarillo eléctrico**: acento principal de marca. Acciones primarias,
//!   selección activa e indicadores importantes.
//! - **Azul técnico**: información eléctrica, datos técnicos, elementos
//!   interactivos secundarios y referencias.
//! - **Morado lavanda**: acento secundario de identidad. Categorías y
//!   elementos complementarios.
//! - **Verde, rojo y naranja**: solo estados semánticos (cumple, no cumple y
//!   aviso), siempre con ícono y texto.
//!
//! Los valores son los de la maqueta aprobada de R2. Los colores que se usan
//! como texto cumplen 4.5 : 1 sobre el panel de su tema (hay una prueba).

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

/// Radio de controles (botones, campos, etiquetas).
pub const RADIUS_CONTROL: u8 = 6;
/// Radio de tarjetas y paneles.
pub const RADIUS_CARD: u8 = 8;
/// Radio de ventanas, menús y hojas.
pub const RADIUS_WINDOW: u8 = 10;

// ---------------------------------------------------- lenguaje visual (R4.5)
// `docs/sistema-visual.md` explica cada valor. Ningún tamaño de letra,
// margen o alto de control se escribe con un número suelto en el código: se
// usan estas constantes (lo comprueba una prueba de este archivo).

/// Escala tipográfica: título de sección o vista.
pub const TITLE: f32 = 22.0;
/// Encabezado: el nombre de lo que se está viendo (tablero, circuito, bloque).
pub const HEAD: f32 = 15.0;
/// Cuerpo: texto, campos, tablas y botones.
pub const BODY: f32 = 13.0;
/// Secundario: aclaraciones, subtítulos y detalle de una cifra.
pub const SECONDARY: f32 = 12.0;
/// Pie: rótulos de cifras y columnas, en tipo oración.
pub const FOOT: f32 = 11.0;
/// Excepción: el título principal de una vista (uno por pantalla).
pub const VIEW_TITLE: f32 = 28.0;
/// Excepción en teléfono: texto de renglones y botones de 44 px.
#[allow(dead_code)] // Lo usan las pantallas de teléfono (R6).
pub const TOUCH: f32 = 15.0;
/// Excepción en teléfono: cifras del resumen.
#[allow(dead_code)] // Lo usan las pantallas de teléfono (R6).
pub const TOUCH_FIGURE: f32 = 17.0;

/// Íconos dentro de texto, tablas y fichas.
pub const ICON: f32 = 14.0;
/// Íconos de la barra superior y de los botones de solo ícono.
pub const ICON_BAR: f32 = 16.0;

/// Retícula de espacios: 4, 8, 12, 16, 24 y 32 px.
pub const SP1: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;
pub const SP4: f32 = 16.0;
pub const SP6: f32 = 24.0;
pub const SP8: f32 = 32.0;

/// Alto de un control en fichas y tablas.
pub const CONTROL_H: f32 = 24.0;
/// Alto de un control en las barras.
pub const BAR_CONTROL_H: f32 = 28.0;
/// Alto de un renglón de tabla o de grupo de campos con ratón.
pub const ROW_H: f32 = 28.0;
/// Alto de un control táctil.
#[allow(dead_code)] // Lo usan las pantallas de teléfono (R6).
pub const TOUCH_H: f32 = 44.0;

/// Paleta de un tema.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    /// Fondo de la aplicación, detrás de los paneles.
    pub bg: Color32,
    /// Superficie de los paneles.
    pub panel: Color32,
    /// Controles y tarjetas.
    pub raise: Color32,
    /// Elemento seleccionado: neutro, en forma de píldora.
    pub selected: Color32,
    pub line: Color32,
    /// Divisor discreto: entre renglones de un grupo y entre regiones.
    pub hair: Color32,
    pub ink: Color32,
    pub ink2: Color32,
    pub ink3: Color32,
    pub ok: Color32,
    /// Rojo de «no cumple» legible como texto. Para rellenos y bordes se usa
    /// `bad_fill`.
    pub bad: Color32,
    /// Rojo para íconos, bordes y rellenos tenues.
    pub bad_fill: Color32,
    /// Advertencias y verificaciones pendientes.
    pub warn: Color32,
    /// Amarillo de marca: barras, bordes, foco y botones rellenos.
    pub accent: Color32,
    /// Acento legible como texto sobre el fondo del tema.
    pub accent_text: Color32,
    /// Texto sobre un botón relleno de acento: oscuro en los dos temas.
    pub on_accent: Color32,
    /// Azul técnico, legible como texto: referencias, datos técnicos y
    /// enlaces.
    pub info: Color32,
    /// Lavanda para puntos, bordes y rellenos tenues.
    pub lav: Color32,
    /// Lavanda legible como texto.
    pub lav_text: Color32,
    /// Fases A, B y C. Siempre se acompañan de su letra.
    pub phases: [Color32; 3],
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        let rgb = Color32::from_rgb;
        if dark {
            Palette {
                dark,
                bg: rgb(0x17, 0x19, 0x1e),
                panel: rgb(0x20, 0x24, 0x2b),
                raise: rgb(0x29, 0x2e, 0x37),
                selected: rgb(0x30, 0x36, 0x40),
                line: rgb(0x36, 0x3c, 0x46),
                hair: rgb(0x2c, 0x31, 0x3a),
                ink: rgb(0xe8, 0xeb, 0xf0),
                ink2: rgb(0xae, 0xb7, 0xc3),
                ink3: rgb(0x8e, 0x99, 0xa8),
                ok: rgb(0x75, 0xb9, 0x9b),
                bad: rgb(0xf0, 0x8a, 0x8a),
                bad_fill: rgb(0xe1, 0x6d, 0x6d),
                warn: rgb(0xe8, 0x92, 0x4a),
                accent: rgb(0xe8, 0xbd, 0x35),
                accent_text: rgb(0xe8, 0xbd, 0x35),
                on_accent: rgb(0x1b, 0x1f, 0x25),
                info: rgb(0x6d, 0x9f, 0xe8),
                lav: rgb(0x9b, 0x91, 0xcc),
                lav_text: rgb(0x9b, 0x91, 0xcc),
                phases: [
                    rgb(0x6d, 0x9f, 0xe8),
                    rgb(0x9b, 0x91, 0xcc),
                    rgb(0x5c, 0xc3, 0xd6),
                ],
            }
        } else {
            Palette {
                dark,
                bg: rgb(0xe9, 0xec, 0xf0),
                panel: rgb(0xf4, 0xf5, 0xf7),
                raise: rgb(0xff, 0xff, 0xff),
                selected: rgb(0xe1, 0xe5, 0xeb),
                line: rgb(0xd3, 0xd8, 0xdf),
                hair: rgb(0xe3, 0xe7, 0xec),
                ink: rgb(0x1b, 0x1f, 0x25),
                ink2: rgb(0x4b, 0x55, 0x60),
                ink3: rgb(0x66, 0x70, 0x7c),
                ok: rgb(0x1f, 0x7a, 0x45),
                bad: rgb(0xc0, 0x30, 0x3a),
                bad_fill: rgb(0xc0, 0x30, 0x3a),
                warn: rgb(0xb4, 0x56, 0x0b),
                accent: rgb(0xe8, 0xbd, 0x35),
                accent_text: rgb(0x7a, 0x5c, 0x00),
                on_accent: rgb(0x1b, 0x1f, 0x25),
                info: rgb(0x2b, 0x66, 0xc4),
                lav: rgb(0x9b, 0x91, 0xcc),
                lav_text: rgb(0x6a, 0x5f, 0xa8),
                phases: [
                    rgb(0x2b, 0x66, 0xc4),
                    rgb(0x6a, 0x5f, 0xa8),
                    rgb(0x0b, 0x72, 0x85),
                ],
            }
        }
    }
}

/// Familia de la tipografía seminegrita.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Registra IBM Plex y los íconos Phosphor. Las fuentes de egui quedan como
/// respaldo para los símbolos que Plex no tiene.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    add(
        "plex-sans",
        include_bytes!("../../../assets/fonts/IBMPlexSans-Regular.ttf"),
    );
    add(
        "plex-sans-semibold",
        include_bytes!("../../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    );
    add(
        "plex-mono",
        include_bytes!("../../../assets/fonts/IBMPlexMono-Regular.ttf"),
    );
    add("phosphor", egui_phosphor::variants::bytes::regular::FONT);

    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let family = |first: &str| {
        let mut list = vec![first.to_owned(), "phosphor".to_owned()];
        list.extend(fallback.iter().cloned());
        list
    };
    fonts
        .families
        .insert(FontFamily::Proportional, family("plex-sans"));
    fonts
        .families
        .insert(FontFamily::Monospace, family("plex-mono"));
    fonts
        .families
        .insert(semibold(), family("plex-sans-semibold"));
    ctx.set_fonts(fonts);
}

fn touch_id() -> egui::Id {
    egui::Id::new("ieestudio-modo-tactil")
}

/// Si la interfaz está en modo táctil: teléfono, o una pantalla que se tocó.
pub fn is_touch(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp(touch_id())).unwrap_or(false)
}

/// Alto de un renglón con el que se interactúa: 28 px con ratón, 44 px
/// táctil.
pub fn row_height(ctx: &egui::Context) -> f32 {
    if is_touch(ctx) {
        TOUCH_H
    } else {
        ROW_H
    }
}

/// Cambia entre la densidad de escritorio y la táctil: controles de 44 px y
/// texto de 15 px (excepción `TOUCH` del sistema visual). Solo hace algo si
/// el modo cambió.
pub fn set_touch(ctx: &egui::Context, touch: bool) {
    if is_touch(ctx) == touch && ctx.data(|d| d.get_temp::<bool>(touch_id())).is_some() {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(touch_id(), touch));
    let (text, control) = if touch {
        (TOUCH, TOUCH_H)
    } else {
        (BODY, CONTROL_H)
    };
    ctx.all_styles_mut(|style| {
        for (kind, family) in [
            (TextStyle::Body, FontFamily::Proportional),
            (TextStyle::Button, FontFamily::Proportional),
            (TextStyle::Monospace, FontFamily::Monospace),
        ] {
            style.text_styles.insert(kind, FontId::new(text, family));
        }
        style.spacing.interact_size.y = control;
        // Con el dedo, arrastrar sobre un texto recorre la lista en lugar de
        // seleccionarlo.
        style.interaction.selectable_labels = !touch;
        style.spacing.button_padding = egui::vec2(if touch { SP3 } else { 10.0 }, SP1);
        style.spacing.item_spacing = egui::vec2(SP2, if touch { SP2 } else { SP1 });
    });
}

/// Aplica la paleta y la densidad de un programa de escritorio profesional.
pub fn apply(ctx: &egui::Context, p: &Palette) {
    ctx.set_theme(if p.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(HEAD, semibold())),
            (TextStyle::Body, FontId::new(BODY, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(BODY, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(BODY, FontFamily::Monospace),
            ),
            (
                TextStyle::Small,
                FontId::new(FOOT, FontFamily::Proportional),
            ),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(SP2, SP1);
        style.spacing.button_padding = egui::vec2(10.0, SP1);
        style.spacing.interact_size.y = CONTROL_H;
        style.spacing.menu_margin = egui::Margin::same(SP2 as i8);
        style.spacing.combo_width = 140.0;

        let v = &mut style.visuals;
        v.dark_mode = p.dark;
        v.override_text_color = Some(p.ink);
        v.panel_fill = p.panel;
        // Lo que flota (ventanas, menús, avisos) es la superficie elevada y
        // lleva la única sombra del sistema.
        v.window_fill = p.raise;
        v.window_stroke = Stroke::new(1.0, p.line);
        let shadow = if p.dark {
            Color32::from_black_alpha(115)
        } else {
            Color32::from_rgba_unmultiplied(15, 23, 42, 36)
        };
        v.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: shadow,
        };
        v.popup_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: shadow,
        };
        v.window_corner_radius = CornerRadius::same(RADIUS_WINDOW);
        v.menu_corner_radius = CornerRadius::same(RADIUS_CARD);
        v.extreme_bg_color = p.bg;
        v.faint_bg_color = p.raise;
        v.code_bg_color = p.raise;
        v.hyperlink_color = p.info;
        v.selection.bg_fill = p.selected;
        v.selection.stroke = Stroke::new(1.0, p.ink);
        v.text_cursor.stroke = Stroke::new(1.5, p.accent);
        v.striped = false;
        let radius = CornerRadius::same(RADIUS_CONTROL);
        for (w, fill, stroke) in [
            (&mut v.widgets.noninteractive, p.panel, p.line),
            (&mut v.widgets.inactive, p.raise, p.line),
            (&mut v.widgets.hovered, p.raise, p.ink3),
            (&mut v.widgets.active, p.raise, p.accent),
            (&mut v.widgets.open, p.raise, p.ink3),
        ] {
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
            w.bg_stroke = Stroke::new(1.0, stroke);
            w.fg_stroke = Stroke::new(1.0, p.ink);
            w.corner_radius = radius;
            w.expansion = 0.0;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.ink2);
        // Separadores y bordes de regiones: el divisor discreto.
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.hair);
    });
    // La paleta nueva trae la densidad de escritorio: se vuelve a la táctil
    // si estaba puesta.
    let touch = is_touch(ctx);
    ctx.data_mut(|d| d.remove::<bool>(touch_id()));
    if touch {
        set_touch(ctx, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(c: Color32) -> f64 {
        let f = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// Todo color que se usa como texto se lee sobre el panel y sobre los
    /// controles de su tema (WCAG AA, 4.5 : 1).
    #[test]
    fn text_colors_meet_contrast_on_panel_and_controls() {
        for dark in [true, false] {
            let p = Palette::new(dark);
            let texts = [
                ("ink", p.ink),
                ("ink2", p.ink2),
                ("ink3", p.ink3),
                ("ok", p.ok),
                ("bad", p.bad),
                ("warn", p.warn),
                ("accent_text", p.accent_text),
                ("info", p.info),
                ("lav_text", p.lav_text),
                ("fase A", p.phases[0]),
                ("fase B", p.phases[1]),
                ("fase C", p.phases[2]),
            ];
            for (name, c) in texts {
                for (surface, bg) in [("panel", p.panel), ("controles", p.raise)] {
                    let r = contrast(c, bg);
                    assert!(
                        r >= 4.5,
                        "{name} sobre {surface} en tema {}: {r:.2}:1",
                        if dark { "oscuro" } else { "claro" }
                    );
                }
            }
        }
    }

    /// El texto de los botones rellenos de acento también se lee.
    #[test]
    fn text_on_accent_meets_contrast() {
        for dark in [true, false] {
            let p = Palette::new(dark);
            assert!(contrast(p.on_accent, p.accent) >= 7.0);
        }
    }

    /// El amarillo de marca no se confunde con el naranja de aviso: están
    /// separados en tono, no solo en luminosidad.
    #[test]
    fn accent_and_warning_stay_apart() {
        for dark in [true, false] {
            let p = Palette::new(dark);
            let hue = |c: Color32| {
                let (r, g, b) = (f64::from(c.r()), f64::from(c.g()), f64::from(c.b()));
                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                if max == min {
                    return 0.0;
                }
                let d = max - min;
                let h = if max == r {
                    ((g - b) / d) % 6.0
                } else if max == g {
                    (b - r) / d + 2.0
                } else {
                    (r - g) / d + 4.0
                };
                (h * 60.0 + 360.0) % 360.0
            };
            assert!((hue(p.accent) - hue(p.warn)).abs() >= 10.0);
        }
    }

    /// Lo escrito en `docs/sistema-visual.md` se cumple en todo el código de
    /// la interfaz: tamaños de letra solo de la escala, márgenes y espacios
    /// de la retícula, y rótulos sin convertir a mayúsculas.
    #[test]
    fn the_interface_uses_only_the_visual_system() {
        let grid = [0, 2, 4, 8, 12, 16, 24, 32];
        let mut problems = Vec::new();
        let mut dirs = vec![std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src"
        ))];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                    continue;
                }
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                // Las pruebas de cada archivo no dibujan nada.
                let code = text.split("#[cfg(test)]").next().unwrap_or("");
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                for (n, line) in code.lines().enumerate() {
                    let at = format!("{name}:{}", n + 1);
                    let after = |key: &str| -> Vec<String> {
                        line.split(key)
                            .skip(1)
                            .map(|r| r.trim_start().to_string())
                            .collect()
                    };
                    for rest in [after(".size("), after("FontId::new(")].concat() {
                        if rest.starts_with(|c: char| c.is_ascii_digit()) {
                            problems.push(format!("{at}: tamaño de letra fuera de la escala"));
                        }
                    }
                    for rest in after("add_space(") {
                        if rest.starts_with(|c: char| c.is_ascii_digit())
                            && !rest.starts_with("2.0)")
                        {
                            problems.push(format!("{at}: espacio fuera de la retícula"));
                        }
                    }
                    for key in ["Margin::symmetric(", "Margin::same("] {
                        for rest in after(key) {
                            let args = rest.split(')').next().unwrap_or("");
                            for value in args.split(',') {
                                if let Ok(v) = value.trim().parse::<i32>() {
                                    if !grid.contains(&v) {
                                        problems
                                            .push(format!("{at}: margen {v} fuera de la retícula"));
                                    }
                                }
                            }
                        }
                    }
                    if line.contains("RichText::new(") && line.contains("to_uppercase()") {
                        problems.push(format!("{at}: rótulo convertido a mayúsculas"));
                    }
                }
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
