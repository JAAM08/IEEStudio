//! Sistema visual de IEEStudio.
//!
//! Gris carbón con superficies apenas diferenciadas, tipografía IBM Plex y un
//! solo color de acento (amarillo eléctrico por defecto) reservado para la
//! marca, el foco y la selección. Los estados tienen colores propios que no
//! se confunden con el acento y siempre van con ícono y texto.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

/// Colores de acento disponibles. Ninguno se parece a los de los estados.
pub const ACCENTS: [(&str, Color32); 3] = [
    ("Amarillo eléctrico", Color32::from_rgb(0xf5, 0xc5, 0x18)),
    ("Azul", Color32::from_rgb(0x4c, 0x9a, 0xff)),
    ("Cian", Color32::from_rgb(0x2e, 0xc4, 0xc4)),
];

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
    /// Elemento seleccionado: neutro, nunca del color de acento.
    pub selected: Color32,
    pub line: Color32,
    pub ink: Color32,
    pub ink2: Color32,
    pub ink3: Color32,
    pub ok: Color32,
    pub bad: Color32,
    /// Advertencias y verificaciones pendientes.
    pub warn: Color32,
    /// Acento para barras, bordes y foco.
    pub accent: Color32,
    /// Acento legible como texto sobre el fondo del tema.
    pub accent_text: Color32,
    /// Texto sobre un botón relleno de acento. Los tres acentos son claros,
    /// así que va oscuro en los dos temas.
    pub on_accent: Color32,
}

impl Palette {
    pub fn new(dark: bool, accent: Color32) -> Self {
        let rgb = Color32::from_rgb;
        if dark {
            Palette {
                dark,
                bg: rgb(0x13, 0x15, 0x18),
                panel: rgb(0x1d, 0x20, 0x24),
                raise: rgb(0x27, 0x2b, 0x31),
                selected: rgb(0x31, 0x37, 0x40),
                line: rgb(0x31, 0x36, 0x3d),
                ink: rgb(0xe4, 0xe7, 0xeb),
                ink2: rgb(0x9a, 0xa2, 0xad),
                ink3: rgb(0x68, 0x70, 0x7b),
                ok: rgb(0x46, 0xc0, 0x6f),
                bad: rgb(0xf4, 0x5d, 0x55),
                warn: rgb(0xec, 0x96, 0x3e),
                accent,
                accent_text: accent,
                on_accent: rgb(0x13, 0x15, 0x18),
            }
        } else {
            Palette {
                dark,
                bg: rgb(0xe1, 0xe4, 0xe8),
                panel: rgb(0xf6, 0xf7, 0xf8),
                raise: rgb(0xff, 0xff, 0xff),
                selected: rgb(0xd9, 0xdf, 0xe7),
                line: rgb(0xcf, 0xd4, 0xda),
                ink: rgb(0x1b, 0x1f, 0x25),
                ink2: rgb(0x55, 0x5e, 0x69),
                ink3: rgb(0x8b, 0x93, 0x9d),
                ok: rgb(0x1a, 0x7f, 0x37),
                bad: rgb(0xc5, 0x22, 0x2c),
                warn: rgb(0xb4, 0x53, 0x09),
                accent,
                accent_text: darken(accent),
                on_accent: rgb(0x13, 0x15, 0x18),
            }
        }
    }
}

/// Versión oscura de un acento, para que se lea como texto en el tema claro.
fn darken(c: Color32) -> Color32 {
    let f = |v: u8| (f32::from(v) * 0.55) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
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

/// Aplica la paleta y la densidad de un programa de escritorio profesional.
pub fn apply(ctx: &egui::Context, p: &Palette) {
    ctx.set_theme(if p.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(15.0, semibold())),
            (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(13.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(12.5, FontFamily::Monospace),
            ),
            (
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            ),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 3.0);
        style.spacing.interact_size.y = 22.0;
        style.spacing.menu_margin = egui::Margin::same(6);
        style.spacing.combo_width = 140.0;

        let v = &mut style.visuals;
        v.dark_mode = p.dark;
        v.override_text_color = Some(p.ink);
        v.panel_fill = p.panel;
        v.window_fill = p.panel;
        v.window_stroke = Stroke::new(1.0, p.line);
        v.window_corner_radius = CornerRadius::same(6);
        v.menu_corner_radius = CornerRadius::same(4);
        v.extreme_bg_color = p.bg;
        v.faint_bg_color = p.raise;
        v.code_bg_color = p.raise;
        v.hyperlink_color = p.accent_text;
        v.selection.bg_fill = p.selected;
        v.selection.stroke = Stroke::new(1.0, p.ink);
        v.text_cursor.stroke = Stroke::new(1.5, p.accent);
        v.striped = false;
        let radius = CornerRadius::same(3);
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
    });
}
