//! Unidades del motor.
//!
//! Todos los valores viajan en unidades del Sistema Internacional usadas en la
//! práctica eléctrica, y los nombres llevan la unidad como sufijo:
//!
//! | Magnitud | Unidad | Sufijo |
//! | --- | --- | --- |
//! | Corriente | ampere | `_a` |
//! | Tensión | volt | `_v` |
//! | Potencia activa | watt | `_w` |
//! | Potencia aparente | voltampere | `_va` |
//! | Longitud de un tramo | metro | `_m` |
//! | Resistencia y reactancia de tablas | ohm por kilómetro | `_ohm_km` |
//! | Área | milímetro cuadrado | `_mm2` |
//! | Temperatura | grado Celsius | `_c` |
//! | Porcentaje | por ciento, 0 a 100 | `_percent` |
//! | Fracción | 0 a 1 | `_fraction` o `factor` |
//!
//! Las conversiones viven aquí para que ninguna fórmula multiplique por una
//! constante sin nombre.

/// Raíz de tres, el factor entre tensión de fase y de línea en un sistema trifásico.
pub const SQRT_3: f64 = 1.732_050_807_568_877_2;

/// Metros a kilómetros, para usar resistencias en ohm por kilómetro.
pub fn m_to_km(length_m: f64) -> f64 {
    length_m / 1000.0
}

/// Valor en por ciento de una fracción.
pub fn percent(value: f64, reference: f64) -> f64 {
    value / reference * 100.0
}
