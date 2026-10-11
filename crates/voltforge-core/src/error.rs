use std::fmt;

/// Motivo por el que un cálculo no puede evaluarse.
///
/// Un error no significa «no cumple»: significa que falta un dato o que la
/// tabla normativa no cubre el caso. La interfaz lo muestra como «no evaluado».
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// El calibre no existe en el perfil normativo.
    UnknownSize(String),
    /// La tabla no tiene valor para la combinación pedida.
    NotInTable { table: &'static str, what: String },
    /// Un dato de entrada no es válido.
    InvalidInput(String),
    /// Ningún tubo de la tabla admite los conductores.
    NoConduitFits { conduit: String, area_mm2: f64 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownSize(s) => {
                write!(f, "el calibre «{s}» no existe en el perfil normativo")
            }
            Error::NotInTable { table, what } => write!(f, "{table} no tiene valor para {what}"),
            Error::InvalidInput(msg) => write!(f, "dato inválido: {msg}"),
            Error::NoConduitFits { conduit, area_mm2 } => write!(
                f,
                "ningún tubo {conduit} de la tabla admite {area_mm2:.2} mm² de conductores"
            ),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
