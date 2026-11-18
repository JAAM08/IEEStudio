# IEEStudio

Cálculo de instalaciones eléctricas de baja tensión en Rust. Antes de cambiar
un criterio de cálculo o una decisión técnica, lee `docs/architecture.md`,
`docs/engineering-methodology.md` y `docs/decisions/`. Lo pendiente está en
`docs/roadmap.md`.

## Comandos

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p ieestudio-cli -- calcular casa.iees
cargo run -p ieestudio-app -- casa.iees
cargo run -p ieestudio-cli -- excel casa.iees        # cuadro de cargas a Excel; «importar» lo trae de vuelta
xvfb-run -a cargo run -p ieestudio-app -- --captura captura.png   # revisar la interfaz sin pantalla
# con --tamano 390x844 para una ventana angosta y --medir 60 para imprimir el tiempo por cuadro
# con --vista proyecto|tableros|unifilar|revisar|memoria, --procedimiento (panel inferior),
# --dividir (cuadro y unifilar a la vez), --propiedades datos|resultados|normativa,
# --claro, --inicio (pantalla de inicio), --comandos (paleta de comandos abierta),
# --seleccion C-03 y --rapido (cálculo rápido); --paso, --espacio, --panel y --avanzada
# son de antes del marco único y se aceptan hasta R7
tools/capturas.sh                                    # vistas × cinco tamaños × dos temas; falla si alguna se cierra
tools/sitio-web.sh                                   # ieestudio.com con la app en el navegador, en target/sitio
cargo clippy --target wasm32-unknown-unknown -p ieestudio-app -- -D warnings
```

Los tres primeros deben pasar antes de cada commit; la integración continua
los ejecuta en Windows y Linux.

## Convenciones

- Interfaz, mensajes, comentarios y documentación en español. Identificadores
  de código en inglés.
- La app también corre en el navegador (WebAssembly): leer y escribir archivos
  siempre por `files`, nunca con `std::fs` o `rfd` directo. Ver la decisión 0017.
- `ieestudio-core` no tiene dependencias obligatorias. Ningún crate de cálculo
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
