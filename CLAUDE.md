# VoltForge

Cálculo de instalaciones eléctricas de baja tensión en Rust. Antes de cambiar
un criterio de cálculo o una decisión técnica, lee `docs/architecture.md`,
`docs/engineering-methodology.md` y `docs/decisions/`. Lo pendiente está en
`docs/roadmap.md`.

## Comandos

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p voltforge-cli -- calcular casa.vfp
cargo run -p voltforge-app -- casa.vfp
xvfb-run -a cargo run -p voltforge-app -- --captura captura.png   # revisar la interfaz sin pantalla
# con --espacio calculo|diagrama|documentacion, --panel rapido|unifilar|memoria|ninguno,
# --propiedades datos|resultados|normativa y --claro
```

Los tres primeros deben pasar antes de cada commit; la integración continua
los ejecuta en Windows y Linux.

## Convenciones

- Interfaz, mensajes, comentarios y documentación en español. Identificadores
  de código en inglés.
- `voltforge-core` no tiene dependencias obligatorias. Ningún crate de cálculo
  (core, rules, electrical, project) conoce la interfaz; ver el orden de
  dependencias en `docs/architecture.md`.
- Ningún valor normativo se escribe en el código: tablas en `data/<perfil>/*.csv`,
  factores y límites en `profile.csv`, cláusulas en `references.csv`. Los
  mensajes citan reglas con `Rule`, nunca con texto como «Tabla 9».
- El motor no redondea. Un caso que las tablas no cubren devuelve un error
  explicativo y se muestra como «no evaluado»; nunca se supone un valor.
- Cada regla de cálculo nueva lleva su `Rule` con referencia y sus tres tipos de
  prueba: matemática, de referencia (de preferencia un caso en `fixtures/`) y
  normativa.
- Nada se marca conforme si falta un dato o una verificación; la aprobación
  profesional nunca la da el programa.
- El dueño del proyecto revisa el funcionamiento, no el código: explica los
  cambios en términos de lo que el programa hace.
