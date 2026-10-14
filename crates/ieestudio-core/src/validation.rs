//! Validación de datos de entrada comunes a varios cálculos.

use crate::error::{Error, Result};

/// El factor de potencia debe ser mayor que 0 y no mayor que 1.
pub fn power_factor(power_factor: f64) -> Result<()> {
    if power_factor > 0.0 && power_factor <= 1.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput(
            "el factor de potencia debe ser mayor que 0 y hasta 1".into(),
        ))
    }
}

/// Un valor físico que no puede ser negativo: potencia, corriente, longitud.
pub fn non_negative(name: &str, value: f64) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput(format!(
            "{name} debe ser un número no negativo"
        )))
    }
}
