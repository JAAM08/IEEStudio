# Rediseño 2026 — R6: tableta y teléfono

**Fecha:** 24 de noviembre de 2026.
**Base:** `main` con R5 integrado (PR #29).
**Referencias:** maqueta R0 (sección 7, composición por tamaño), mesa de
teléfono aprobada en R4.5 (`TelefonoR45`) y `docs/sistema-visual.md`.

R6 arma el marco de cada tamaño y la pantalla de inicio con dos experiencias.
No toca el motor, los cálculos, las tablas, el formato `.iees`, el contenido
de la memoria en PDF ni el dibujo del unifilar.

## 1. Pantalla de inicio: dos experiencias independientes

Requisito del dueño del proyecto del 18 de noviembre de 2026.

- **Proyecto completo:** «Nuevo proyecto» (la acción amarilla), «Abrir…»,
  «Ejemplo» y los recientes (en escritorio).
- **Cálculo rápido:** abre las herramientas de cálculo directo **en su propia
  pantalla**, sin proyecto: sin barra de vistas, sin barra de estado del
  proyecto y sin «Agregar al proyecto». Su acción principal es «Exportar hoja
  PDF». «‹ Inicio» regresa.
- En escritorio las dos tarjetas van lado a lado; desde 784 px hacia abajo,
  una debajo de otra.
- Dentro de un proyecto en escritorio, F8 sigue abriendo el cálculo rápido
  como ventana encima, con «Agregar al proyecto». En el teléfono, F8 y el
  menú abren la pantalla propia.

## 2. Tres marcos según el ancho

| Clase | Ancho | Navegación | Navegador | Ficha |
|---|---|---|---|---|
| Escritorio | 900 px o más | Barra de vistas arriba | Fijo | Fija a la derecha |
| Tableta | 600 a 899 px | Barra de vistas arriba, solo íconos | Cajón que se abre encima | Cajón a la derecha |
| Teléfono | menos de 600 px | Barra inferior con cinco destinos | Lista de tableros | Pantalla completa |

### Teléfono

- **Barra inferior** con Proyecto, Circuitos, Unifilar, Revisar y Memoria;
  una cápsula detrás del ícono activo y la insignia de pendientes en Revisar,
  como en la mesa aprobada.
- **Barra superior corta:** el nombre del programa o «‹ atrás» (con el
  tablero al que se vuelve), búsqueda y un menú con deshacer, rehacer,
  cálculo rápido, inicio, tema, Archivo y Ayuda. Sin barra de estado: el
  veredicto está en Proyecto, en Revisar y en cada ficha. Los avisos salen en
  un renglón arriba de los destinos.
- **Tableros y circuitos:** lista de tableros → tarjetas de circuito del
  tablero (ID, descripción, estado, carga, calibre, interruptor y caída
  acumulada) → ficha a pantalla completa. «Agregar circuito» y «Ficha del
  tablero» arriba de las tarjetas. Un circuito nuevo abre su ficha.
- **Unifilar:** el dibujo completo con sus controles flotantes; tocar un
  tablero o circuito abre una **hoja inferior** con su estado, sus cifras, lo
  que no cumple y «Abrir ficha».
- **Proyecto:** los tableros pasan de tabla a lista que se toca.
- **Memoria:** la de R5 (ficha plegada arriba, la hoja al ancho).
- **Revisar:** la misma de antes; se rehace en R7 con el retiro de
  `guided.rs`.

### Tableta

- **Tableros y circuitos** muestra el cuadro de cargas completo (con el juego
  «Esencial» o «Completo»), ya no los contenidos «Tableros» y «Circuitos» de
  la vista guiada.
- **Cajones:** el navegador sale de la izquierda con el botón de barra
  lateral; la ficha, de la derecha, sola al elegir un tablero o circuito en el
  cuadro, el unifilar o el navegador, o con su botón. Tocar fuera cierra. Se
  deslizan en 150 ms.
- La barra superior deja las cinco vistas solo con su ícono y quita el nombre
  del proyecto (está en la barra de estado), para que todo quepa sin
  recorrerse de lado.

### Modo táctil

- En el teléfono siempre; en otros tamaños, desde que la pantalla se toca
  hasta que se vuelve a mover el ratón.
- Controles y renglones de 44 px y texto de 15 px (excepciones `TOUCH_H` y
  `TOUCH` del sistema visual). Las fichas, el cuadro y el navegador usan esa
  altura.
- Los textos no se seleccionan arrastrando: arrastrar recorre.
- En la versión web, el lienzo pide al navegador no recorrer ni ampliar la
  página por su cuenta (`touch-action: none`): los gestos los maneja la app.

### Ventanas que se ajustan

- La paleta de comandos nunca es más ancha que la ventana.
- «¿Qué vas a conectar?» pone de una a cuatro tarjetas por renglón según el
  ancho, con desplazamiento si no caben a lo alto. Sus textos ya no se pueden
  seleccionar: tocar o hacer clic en el nombre de la tarjeta la elige (antes,
  en el texto no respondía).
- El cálculo rápido pone todo en una columna en ventanas angostas.

## 3. Validación funcional de navegación y desplazamiento

Versión web compilada con `tools/sitio-web.sh`, abierta en Chromium sin
pantalla con un teléfono emulado (Pixel 7, 412 × 839, táctil, escala 1) y
manejada con **toques y gestos táctiles reales del navegador** (eventos
`touchstart`, `touchmove` y `touchend`, uno o dos dedos). Cada paso dejó una
captura que se revisó:

| Prueba | Resultado |
|---|---|
| Tocar una tarjeta de circuito abre su ficha | Correcto |
| Deslizar la ficha la recorre | Correcto |
| «‹ TG-1» vuelve a las tarjetas; «‹ Tableros», a la lista | Correcto |
| Los cinco destinos de la barra inferior | Correcto |
| Pellizcar el unifilar acerca sobre los dedos (39 % → 140 %) | Correcto |
| Arrastrar el unifilar lo recorre | Correcto |
| Deslizar Revisar recorre el dictamen y las incidencias | Correcto |
| Deslizar Memoria pasa a la página 2 y convierte solo las nuevas | Correcto |
| Menú → Cálculo rápido → deslizar → «‹ Inicio» | Correcto |
| «Agregar circuito» → tocar «Alumbrado» → ficha del circuito nuevo | Correcto (después de la corrección de los textos que no se seleccionan) |
| Tocar un circuito en el unifilar → hoja inferior → «Abrir ficha» | Correcto |

Sin errores en la consola de la página. Chromium avisaba que el programa no
podía detener el desplazamiento del navegador en los gestos; con
`touch-action: none` en el lienzo el navegador ya no los toma.

**Lo que esta validación no cubre:** un teléfono real (iOS Safari no se
probó), el teclado en pantalla (se midió en R1) y la sensación de los gestos
con la mano.

## 4. Pruebas

Nuevas:

- `each_width_has_its_frame`: escritorio, tableta y teléfono por ancho; el
  cajón nunca tapa toda la ventana ni queda negativo.
- `the_phone_goes_from_panels_to_circuits_to_the_ficha_and_back`.
- `the_circuit_card_shows_load_cable_breaker_and_drop`.
- `the_worst_status_wins`.
- `the_five_destinations_have_short_names`.
- `the_two_experiences_stack_on_narrow_windows`.
- `the_tiles_fit_the_window` («¿Qué vas a conectar?»).

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para `wasm32`
y `cargo test` pasan: **303 pruebas, 0 fallas**. Ninguna prueba de cálculo
cambió. La prueba del sistema visual pasa con el código nuevo.

`tools/capturas.sh` suma tres vistas: `inicio`, `rapido` y `ficha` (la del
circuito abierta: pantalla completa en teléfono, cajón en tableta). La app
acepta `--ficha`.

## 5. Capturas

En `docs/capturas-r6/`, lado a lado R5 (`main`) y R6, mismo tamaño y tema:

- Inicio a 1440 × 900 y a 390 × 844.
- Proyecto, Tableros y circuitos y Unifilar a 390 × 844.
- Tableros y circuitos a 768 × 1024 (tableta).
- Nuevas, sin equivalente en R5 (`nuevo-…`): la ficha a pantalla completa en
  teléfono, la ficha en su cajón en tableta y la pantalla de cálculo rápido
  en teléfono.

Matriz completa de `tools/capturas.sh` (compilación optimizada), con las tres
vistas nuevas: **100 de 100 capturas sin fallas** (diez vistas, cinco tamaños,
dos temas). Además, **40 de 40** en compilación de depuración en los cuatro
tamaños angostos, para que un ancho negativo cierre el programa si lo
hubiera.

## 6. Rendimiento

Compilación optimizada, dibujo por software. `main` (R5) y R6 se midieron
alternados, dos corridas cada uno, a 1440 × 900:

| Proyecto | Vista | Mediana R5 | Mediana R6 | p95 R5 | p95 R6 |
|---|---|---|---|---|---|
| casa | proyecto | 0.43 · 0.42 ms | 0.39 · 0.42 ms | 0.57 · 0.60 | 0.47 · 0.54 |
| casa | cuadro | 1.14 · 1.19 ms | 1.13 · 1.10 ms | 1.79 · 1.72 | 1.60 · 1.48 |
| casa | unifilar | 0.77 · 0.74 ms | 0.76 · 0.73 ms | 1.11 · 1.10 | 1.16 · 1.09 |
| casa | revisar | 0.64 · 0.63 ms | 0.65 · 0.63 ms | 0.87 · 0.85 | 1.02 · 0.97 |
| grande | proyecto | 0.87 · 0.84 ms | 0.86 · 0.84 ms | 1.36 · 1.15 | 0.99 · 1.13 |
| grande | cuadro | 4.06 · 3.93 ms | 3.93 · 3.90 ms | 5.55 · 5.56 | 4.99 · 5.30 |
| grande | unifilar | 3.47 · 3.40 ms | 3.40 · 3.43 ms | 5.06 · 4.79 | 3.88 · 4.91 |
| grande | revisar | 9.67 · 9.98 ms | 9.80 · 9.78 ms | 12.41 · 13.21 | 12.63 · 11.95 |

En escritorio no cambia: las diferencias están dentro del ruido. La memoria
sube 3 a 4 MiB (de 161–184 a 164–188 MiB).

En teléfono (R6, 390 × 844, una corrida):

| Proyecto | Proyecto | Circuitos | Unifilar | Revisar | Memoria |
|---|---|---|---|---|---|
| casa: mediana / p95 | 0.20 / 0.29 ms | 0.14 / 0.22 ms | 0.18 / 0.27 ms | 0.36 / 0.72 ms | 0.09 / 0.15 ms |
| grande: mediana / p95 | 0.55 / 0.72 ms | 0.36 / 0.54 ms | 0.88 / 1.18 ms | 8.78 / 10.88 ms | 0.25 / 0.47 ms |

La lista de circuitos cuesta menos que el cuadro de escritorio (0.36 contra
3.9 ms con el proyecto grande). Memoria con 522 páginas en teléfono: 535 MiB
y un cuadro de 1.4 s al componer, como en escritorio (R5).

## 7. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (4 min 47 s). El
módulo pesa 50,909,200 bytes contra 50,781,449 de R5: 127,751 bytes más
(0.25 %). La sección 3 se hizo sobre este sitio.

## 8. Límites y lo que queda

- Revisar en teléfono es la de antes (funciona y se recorre); sus tarjetas de
  incidencia y contadores en dos columnas se hacen en R7, al sacarla de
  `guided.rs`.
- Los contenidos «Tableros» y «Circuitos» de `guided.rs` ya no se muestran en
  ningún tamaño; se retiran en R7 junto con el resto del archivo. La opción
  `--paso tableros` ahora solo elige la vista.
- El aviso de «pantallas grandes» de la versión web sigue; se quita en R7.
- No se probó en un teléfono ni una tableta reales.
