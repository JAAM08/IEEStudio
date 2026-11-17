//! Medidas que dependen del ancho de la ventana.
//!
//! egui no revisa que un ancho calculado sea positivo: con una ventana angosta
//! una resta como «lo disponible menos el panel lateral» queda negativa, y la
//! compilación de depuración se cierra con «Negative width makes no sense».
//! Aquí están las cuentas, sin interfaz, para probarlas solas.

/// Ancho mínimo que necesita el contenido principal de un paso junto a su panel
/// lateral fijo. Con menos, el panel lateral pasa abajo (ver [`side_fits`]).
pub const MIN_CONTENT: f32 = 420.0;

/// Ancho mínimo de una tarjeta para ponerla junto a otras en una fila.
pub const MIN_CARD: f32 = 150.0;

/// Lo que queda de `available` después de quitarle `used`, nunca negativo.
pub fn remaining(available: f32, used: f32) -> f32 {
    (available - used).max(0.0)
}

/// Ancho de cada una de `n` columnas iguales separadas por `gap`, nunca negativo.
pub fn column_width(available: f32, n: usize, gap: f32) -> f32 {
    let n = n.max(1) as f32;
    remaining(available, gap * (n - 1.0)) / n
}

/// Si un panel lateral de ancho `side` cabe junto al contenido principal.
/// Si no cabe, el paso lo muestra abajo, a lo ancho, en lugar de encimarlo
/// sobre el contenido o dejar a este con ancho negativo.
pub fn side_fits(available: f32, side: f32) -> bool {
    available >= side + MIN_CONTENT
}

/// Cuántas de `n` tarjetas caben en una fila: todas si cada una mide al menos
/// [`MIN_CARD`]; si no, una debajo de otra.
pub fn cards_per_row(available: f32, n: usize, gap: f32) -> usize {
    if n > 1 && column_width(available, n, gap) >= MIN_CARD {
        n
    } else {
        1
    }
}

/// Ancho de ventana desde el que las barras de arriba y de abajo caben en un
/// renglón como en escritorio. Es el corte de 900 px entre tableta y
/// escritorio compacto de la maqueta R0. Con menos, la barra de estado pasa el
/// veredicto a un segundo renglón y la barra superior de la vista guiada se
/// recorre de lado.
pub const WIDE: f32 = 900.0;

/// Si la ventana es lo bastante ancha para las barras en un renglón.
pub fn bars_in_one_row(width: f32) -> bool {
    width >= WIDE
}

/// Alto inicial y límites del panel que se muestra abajo cuando no cabe al
/// lado: un poco menos de la mitad de la ventana, sin tapar todo el contenido.
pub fn bottom_panel(height: f32) -> (f32, f32, f32) {
    let max = (height * 0.75).max(120.0);
    let min = 120.0_f32.min(max);
    let default = (height * 0.45).clamp(min, max);
    (default, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_is_never_negative() {
        assert_eq!(remaining(100.0, 30.0), 70.0);
        assert_eq!(remaining(10.0, 30.0), 0.0);
        assert_eq!(remaining(-5.0, 0.0), 0.0);
    }

    #[test]
    fn columns_share_the_width_without_going_negative() {
        assert_eq!(column_width(312.0, 3, 12.0), 96.0);
        assert_eq!(column_width(20.0, 3, 12.0), 0.0);
        assert_eq!(column_width(100.0, 0, 12.0), 100.0);
        for available in [0.0, 1.0, 50.0, 390.0 - 64.0, 1440.0] {
            for n in 0..5 {
                assert!(column_width(available, n, 12.0) >= 0.0);
            }
        }
    }

    // El caso que cerraba el paso Revisar: una ventana de 600 px menos el
    // panel del unifilar (440 px) y los márgenes (64 px) dejaba 96 px para
    // tres tarjetas, y cada una pedía (96 − 24) / 3 − 4 − 30 = −10 px.
    #[test]
    fn review_cards_stack_on_narrow_windows() {
        // Escritorio: igual que antes, las tres en una fila.
        assert_eq!(cards_per_row(1440.0 - 440.0 - 64.0, 3, 12.0), 3);
        assert_eq!(cards_per_row(1024.0 - 440.0 - 64.0, 3, 12.0), 3);
        // Sin el panel al lado, 600 px aún da para tres; 390 px, una por fila.
        assert_eq!(cards_per_row(600.0 - 64.0, 3, 12.0), 3);
        assert_eq!(cards_per_row(390.0 - 64.0, 3, 12.0), 1);
        assert_eq!(cards_per_row(96.0, 3, 12.0), 1);
        assert_eq!(cards_per_row(5000.0, 1, 12.0), 1);
    }

    #[test]
    fn side_panels_move_below_when_they_do_not_fit() {
        // Paneles fijos de los pasos: Tableros 420, Circuitos 390, Revisar y
        // Memoria 440.
        for side in [390.0, 420.0, 440.0] {
            assert!(side_fits(1440.0, side));
            assert!(side_fits(1024.0, side));
            assert!(!side_fits(768.0, side));
            assert!(!side_fits(600.0, side));
            assert!(!side_fits(390.0, side));
        }
    }

    #[test]
    fn bars_change_below_the_tablet_breakpoint() {
        assert!(bars_in_one_row(1440.0));
        assert!(bars_in_one_row(1024.0));
        assert!(bars_in_one_row(900.0));
        assert!(!bars_in_one_row(768.0));
        assert!(!bars_in_one_row(390.0));
    }

    #[test]
    fn bottom_panel_leaves_room_for_the_content() {
        let (default, min, max) = bottom_panel(844.0);
        assert!(min <= default && default <= max);
        assert!(max < 844.0);
        let (default, min, max) = bottom_panel(100.0);
        assert!(min <= default && default <= max);
    }
}
