# VoltForge

Cálculo de instalaciones eléctricas de baja tensión en Rust. Antes de cambiar
un criterio de cálculo o una decisión técnica, lee `docs/decisiones.md`.

## Comandos

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p voltforge-cli -- calcular casa.json
cargo run -p voltforge-app -- casa.json
xvfb-run -a cargo run -p voltforge-app -- --captura captura.png   # revisar la interfaz sin pantalla
```

Los tres primeros deben pasar antes de cada commit; la integración continua
los ejecuta en Windows y Linux.

## Convenciones

- Interfaz, mensajes, comentarios y documentación en español. Identificadores
  de código en inglés.
- `voltforge-core` no tiene dependencias obligatorias y no conoce la interfaz.
- Ningún valor normativo se escribe en el código: va en `data/<perfil>/*.csv`
  con su origen en el README de esa carpeta.
- El motor no redondea. Un caso que las tablas no cubren devuelve un error
  explicativo y se muestra como «no evaluado»; nunca se supone un valor.
- Cada regla de cálculo nueva lleva una prueba con resultado calculado a mano
  y la referencia a la tabla o artículo en un comentario.
- El dueño del proyecto revisa el funcionamiento, no el código: explica los
  cambios en términos de lo que el programa hace.
