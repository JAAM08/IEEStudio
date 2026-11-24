# 0034. Un solo marco adaptativo para escritorio, tableta y teléfono

**Estado:** aceptada (24 de noviembre de 2026). El dueño del proyecto aprobó
el plan del rediseño 2026 (`../rediseno-2026-diagnostico-y-plan.md`), la
opción A de la decisión D-3 (regiones fijas, sin paneles acoplables), el
lenguaje visual de R4.5 y cada etapa de R1 a R6. Sustituye a la **0010** y a
la **0020**; la **0019** queda como antecedente.

## Contexto

Hasta el rediseño, IEEStudio tenía dos interfaces sobre el mismo proyecto:
la vista guiada en cinco pasos (0020) y la avanzada con paneles acoplables
(0010, 0019). Cada función se programaba dos veces, el teléfono usaba la
guiada apretada y el dueño del proyecto pidió una sola experiencia moderna,
coherente y sin perder densidad de información.

## Decisión

- **Un solo marco.** Barra superior con las cinco vistas (Proyecto, Tableros
  y circuitos, Unifilar, Revisar, Memoria), la vista al centro y, alrededor,
  regiones fijas: navegador, ficha (inspector) y procedimiento abajo. Sin
  paneles acoplables ni modo guiado o avanzado.
- **Tres clases de tamaño** (`layout::class`): escritorio desde 900 px,
  tableta de 600 a 899 px (navegador y ficha en cajones) y teléfono por
  debajo de 600 px (barra inferior de cinco destinos, listas y ficha a
  pantalla completa).
- **Modo táctil:** controles de 44 px y texto de 15 px en el teléfono o
  cuando la pantalla se toca.
- **Dos experiencias desde el inicio:** «Proyecto completo» y «Cálculo
  rápido», esta en su propia pantalla, sin proyecto.
- **Un solo `Doc`, un solo cálculo:** todas las vistas y tamaños editan y
  muestran lo mismo; cambiar de vista o de tamaño nunca recalcula.
- **Sistema visual obligatorio** (`../sistema-visual.md`), revisado por una
  prueba.

## Consecuencias

- Se retiraron `guided.rs`, la dependencia `egui_dock`, la preferencia
  `guided`, el espacio de trabajo guardado y las opciones `--paso`,
  `--espacio`, `--panel`, `--avanzada`, `--unifilar` y
  `--unifilar-completo`. Las preferencias guardadas por versiones anteriores
  siguen cargando: lo que ya no existe se ignora.
- Una función nueva se programa una vez y se acomoda por tamaño; no hay dos
  interfaces que mantener.
- Detalle de cada etapa: `../rediseno-2026-r1-linea-base.md` a
  `../rediseno-2026-r7-retiro-y-cierre.md`.
