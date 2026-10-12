# Cómo contribuir

VoltForge calcula instalaciones eléctricas. Un error de cálculo puede terminar
en una instalación insegura, así que toda contribución sigue estas reglas.

## Antes de empezar

- Lee `docs/architecture.md` (qué va en cada crate) y `docs/decisions/` (lo
  que ya se decidió y por qué).
- Para cambiar un criterio de cálculo, abre primero un *issue* que explique el
  criterio, la cláusula de la norma y un caso de ejemplo resuelto a mano.

## Reglas para el código de cálculo

1. **Ningún valor normativo en el código.** Tablas, factores, límites y
   calibres mínimos van en `data/<perfil>/` (CSV) con su origen en el README
   de esa carpeta. Las cláusulas van en `data/<perfil>/references.csv`.
2. **Cada regla nueva lleva su referencia** (`Rule` y una fila en
   `references.csv`) y sus tres tipos de prueba: matemática, de referencia y
   normativa (ver `docs/engineering-methodology.md`).
3. **El motor no redondea.** El redondeo es asunto de quien presenta.
4. **Nada se supone.** Si una tabla no cubre un caso, se devuelve un error que
   lo explica y el elemento queda como «no evaluado».
5. **El programa nunca aprueba.** Nada se marca conforme si falta un dato o
   una verificación.

## Antes de cada commit

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

La integración continua ejecuta lo mismo en Windows y Linux.

## Casos de ingeniería

Para aportar un caso sin escribir Rust, agrega una fila a los CSV de
`fixtures/engineering-cases/` con el cálculo hecho a mano y su fuente. Si eres
ingeniero y revisaste un caso existente, anota tu nombre en `reviewed_by`.

## Idioma

Interfaz, mensajes, comentarios y documentación en español. Identificadores de
código en inglés.

## Licencia

Al contribuir aceptas que tu aporte se publique bajo la doble licencia del
proyecto, MIT o Apache-2.0, a elección de quien lo use.
