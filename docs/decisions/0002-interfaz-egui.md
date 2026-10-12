# 0002. Interfaz con egui

**Estado:** aceptada. Se reconsideró Slint en la auditoría de octubre de 2026
y se mantuvo egui.

## Contexto

La interfaz debe ser profesional, con temas y colores personalizables, y el
mismo código debería servir en escritorio y navegador.

## Decisión

egui con eframe. La interfaz ya construida (proyecto, unifilar y cálculo
rápido) funciona en Windows y Linux.

## Alternativas

- **Slint:** archivos `.slint` declarativos y buen soporte de escritorio;
  cambiar implicaría reconstruir todas las pantallas.

## Consecuencias

- Modo inmediato: la vista se redibuja desde el modelo en cada cuadro, lo que
  simplifica mantener sincronizados el cuadro de cargas, el detalle y el
  unifilar.
- Se compila a WebAssembly para la versión en navegador.
- Las pantallas se verifican sin monitor con `xvfb-run` y `--captura`.
