//! Lector mínimo de CSV para las tablas incrustadas.
//!
//! Admite campos entre comillas con comas y comillas dobladas, que es todo lo
//! que usan los archivos de `data/`. Evita depender de bibliotecas externas.

use std::collections::HashMap;
use std::str::FromStr;

/// Una fila, con acceso a sus campos por nombre de columna.
pub struct Row<'a> {
    table: &'a str,
    line: usize,
    columns: &'a HashMap<String, usize>,
    fields: Vec<String>,
}

impl Row<'_> {
    fn raw(&self, column: &str) -> &str {
        let i = *self
            .columns
            .get(column)
            .unwrap_or_else(|| panic!("{}: falta la columna «{column}»", self.table));
        self.fields.get(i).map(String::as_str).unwrap_or("").trim()
    }

    /// Texto del campo.
    pub fn text(&self, column: &str) -> String {
        self.raw(column).to_string()
    }

    /// Texto del campo, o `None` si está vacío.
    pub fn opt_text(&self, column: &str) -> Option<String> {
        let s = self.raw(column);
        (!s.is_empty()).then(|| s.to_string())
    }

    /// Valor del campo, o `None` si está vacío.
    pub fn opt<T: FromStr>(&self, column: &str) -> Option<T> {
        let s = self.raw(column);
        if s.is_empty() {
            return None;
        }
        Some(s.parse().unwrap_or_else(|_| {
            panic!(
                "{} línea {}: «{s}» no es válido en «{column}»",
                self.table, self.line
            )
        }))
    }

    /// Valor obligatorio del campo.
    pub fn get<T: FromStr>(&self, column: &str) -> T {
        self.opt(column)
            .unwrap_or_else(|| panic!("{} línea {}: «{column}» está vacío", self.table, self.line))
    }
}

fn split(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    fields.push(field);
    fields
}

/// Convierte cada fila de datos con `build`. La primera línea es el encabezado.
pub fn parse<T>(table: &str, text: &str, build: impl Fn(&Row) -> T) -> Vec<T> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty());
    let (_, header) = lines
        .next()
        .unwrap_or_else(|| panic!("{table}: archivo vacío"));
    let columns: HashMap<String, usize> = split(header)
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name.trim().to_string(), i))
        .collect();
    lines
        .map(|(i, line)| {
            build(&Row {
                table,
                line: i + 1,
                columns: &columns,
                fields: split(line),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_fields_keep_commas() {
        let rows = parse("t", "a,b\n1,\"x, \"\"y\"\"\"\n2,\n", |r| {
            (r.get::<u32>("a"), r.opt_text("b"))
        });
        assert_eq!(rows, vec![(1, Some("x, \"y\"".to_string())), (2, None)]);
    }
}
