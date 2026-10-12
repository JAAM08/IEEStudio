//! Sistema visual de VoltForge: paleta, tipografía y estilo de los controles.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

/// Colores de acento disponibles.
pub const ACCENTS: [(&str, Color32); 4] = [
    ("Ámbar", Color32::from_rgb(0xf2, 0xb2, 0x33)),
    ("Azul", Color32::from_rgb(0x4a, 0xa8, 0xff)),
    ("Verde", Color32::from_rgb(0x3f, 0xc7, 0xa4)),
    ("Naranja", Color32::from_rgb(0xe8, 0x79, 0x4a)),
];

/// Paleta de un tema.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub raise: Color32,
    pub line: Color32,
    pub ink: Color32,
    pub ink2: Color32,
    pub ok: Color32,
    pub bad: Color32,
    pub na: Color32,
    pub accent: Color32,
}

impl Palette {
    pub fn new(dark: bool, accent: Color32) -> Self {
        let rgb = Color32::from_rgb;
        if dark {
            Palette {
                dark,
                bg: rgb(0x12, 0x16, 0x1b),
                panel: rgb(0x19, 0x1e, 0x25),
                raise: rgb(0x22, 0x29, 0x32),
                line: rgb(0x2c, 0x34, 0x3f),
                ink: rgb(0xe7, 0xea, 0xef),
                ink2: rgb(0x9b, 0xa6, 0xb5),
                ok: rgb(0x4c, 0xc3, 0x8a),
                bad: rgb(0xff, 0x8a, 0x5c),
                na: rgb(0xd8, 0xb0, 0x4a),
                accent,
            }
        } else {
            Palette {
                dark,
                bg: rgb(0xf1, 0xf3, 0xf6),
                panel: rgb(0xff, 0xff, 0xff),
                raise: rgb(0xe9, 0xed, 0xf2),
                line: rgb(0xd5, 0xdb, 0xe3),
                ink: rgb(0x18, 0x20, 0x2a),
                ink2: rgb(0x56, 0x62, 0x73),
                ok: rgb(0x0f, 0x7a, 0x4b),
                bad: rgb(0xb8, 0x40, 0x1a),
                na: rgb(0x7a, 0x5a, 0x00),
                accent,
            }
        }
    }
}

/// Familia de la tipografía seminegrita.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Registra IBM Plex. Las fuentes incluidas con egui quedan como respaldo
/// para los símbolos que Plex no tiene.
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

    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let with_fallback = |first: &str| {
        let mut list = vec![first.to_owned()];
        list.extend(fallback.iter().cloned());
        list
    };
    fonts
        .families
        .insert(FontFamily::Proportional, with_fallback("plex-sans"));
    fonts
        .families
        .insert(FontFamily::Monospace, with_fallback("plex-mono"));
    fonts
        .families
        .insert(semibold(), with_fallback("plex-sans-semibold"));
    ctx.set_fonts(fonts);
}

/// Aplica la paleta a todos los controles.
pub fn apply(ctx: &egui::Context, p: &Palette) {
    ctx.set_theme(if p.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(18.0, semibold())),
            (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(13.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
            (
                TextStyle::Small,
                FontId::new(11.5, FontFamily::Proportional),
            ),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;

        let v = &mut style.visuals;
        v.dark_mode = p.dark;
        v.override_text_color = Some(p.ink);
        v.panel_fill = p.panel;
        v.window_fill = p.panel;
        v.window_stroke = Stroke::new(1.0, p.line);
        v.extreme_bg_color = p.bg;
        v.faint_bg_color = p.raise;
        v.hyperlink_color = p.accent;
        v.selection.bg_fill = p.accent.gamma_multiply(if p.dark { 0.28 } else { 0.35 });
        v.selection.stroke = Stroke::new(1.0, if p.dark { p.accent } else { p.ink });
        let radius = CornerRadius::same(5);
        for (w, fill, stroke) in [
            (&mut v.widgets.noninteractive, p.panel, p.line),
            (&mut v.widgets.inactive, p.raise, p.line),
            (&mut v.widgets.hovered, p.raise, p.ink2),
            (&mut v.widgets.active, p.raise, p.accent),
            (&mut v.widgets.open, p.raise, p.accent),
        ] {
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
            w.bg_stroke = Stroke::new(1.0, stroke);
            w.fg_stroke = Stroke::new(1.0, p.ink);
            w.corner_radius = radius;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.ink2);
    });
}
