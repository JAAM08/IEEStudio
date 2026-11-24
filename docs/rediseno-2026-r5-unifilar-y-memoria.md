# Rediseño 2026 — R5: unifilar y memoria

**Fecha:** 24 de noviembre de 2026.
**Base:** `main` con R4.5 integrado (PR #28).
**Sistema visual:** `docs/sistema-visual.md` (obligatorio desde R4.5).

R5 cambia cómo se ven y se recorren el unifilar y la memoria de cálculo. No
toca el motor, los cálculos, las tablas, el formato `.iees`, el contenido de
la memoria en PDF ni el dibujo del unifilar: el PDF que se exporta es el mismo
documento, y el unifilar dibuja los mismos elementos.

## 1. Qué cambia para quien usa el programa

### Memoria

- **Ya no se convierte todo el documento en imágenes.** La memoria se compone
  una vez y solo se convierten en imagen las páginas que se ven, más una de
  cada lado. Como mucho hay **seis imágenes de página en memoria**, sin importar
  si el documento tiene 13 o 522 páginas. Al desplazarse se convierten a lo más
  dos páginas por cuadro; mientras tanto se ve la hoja en blanco en su lugar.
- **El proyecto de 386 circuitos ya abre en Memoria.** Antes pasaba de 5.4 GB
  y la prueba se detenía; ahora usa unos 550 MB (ver §3).
- **«Componiendo la memoria…»** aparece un instante antes de componer, para
  que la ventana no se quede quieta sin aviso (sobre todo en el navegador).
- **Se compone al entrar a Memoria o con «Actualizar»,** nunca con cada
  tecla. Si el proyecto cambia mientras se está en la vista, aparece el aviso
  «El proyecto cambió: actualiza la vista previa».
- **Exportar PDF es el botón amarillo** de la vista y, si la vista previa ya
  compuso esa misma versión del proyecto, el PDF sale de ahí sin componer otra
  vez.
- La barra dice en qué página se está: «Página 3 de 13 · 3 imágenes en
  memoria».
- **La nitidez sigue al zoom:** al acercar, las páginas se vuelven a convertir
  con más detalle (hasta 2 píxeles por punto).
- Cada hoja se centra en la ventana; la que no cabe (la del unifilar,
  apaisada) empieza a la izquierda y se recorre. En ventanas angostas el zoom
  inicial ajusta la hoja al ancho.

### Ficha de Memoria

La columna de la izquierda pasó a ser una **ficha a la derecha**, como el
inspector de las otras vistas, en grupos de campos:

- **Firma:** Responsable, Cédula, Revisión y Fecha, en renglones de 28 px con
  divisores finos.
- **Antes de entregar:** los mismos cuatro puntos de antes (tramos
  calculados, nada incumple, nada sin evaluar, responsable y cédula) y lo que
  queda fuera de alcance, cada uno con ícono, texto y color.
- **También puedes exportar:** Excel, DXF y «Ver el unifilar».

En ventanas angostas (menos de 900 px) la ficha queda arriba, plegada bajo
«Firma y entrega», y la vista previa ocupa el resto.

### Unifilar

- **Controles flotantes en cápsula**, arriba a la derecha del dibujo, con la
  sombra de lo flotante: Revisión o Plano, alejar, zoom, acercar y ajustar al
  ancho. La barra de arriba del dibujo desapareció.
- **Acercar con Ctrl y la rueda (Cmd en Mac) o pellizcando**, sobre el punto
  del cursor: lo que está bajo el cursor se queda ahí.
- **Arrastrar para recorrer** el dibujo con el ratón (antes solo con la
  rueda o las barras). Un clic sin arrastrar sigue eligiendo el tablero o
  circuito.
- **Solo se dibuja lo que se ve.** Con cientos de circuitos, acomodar el texto
  de lo que estaba fuera de la ventana era el costo principal del cuadro.
- En ventanas angostas la indicación «Haz clic en un tablero…» se oculta para
  que «Exportar DXF» quepa.

## 2. Lo que se retiró

- El contenido «Memoria» de la vista guiada (`guided::report_step`) y su paso
  `Step::Report`: la vista Memoria tiene su propio módulo (`panels/report.rs`).
- La función que devolvía todas las páginas como imágenes
  (`ieestudio_reports::project_pages`). En su lugar, `compose_project` devuelve
  la memoria compuesta, que sabe cuántas páginas tiene, su tamaño, convertir
  una sola página en imagen y dar el PDF.

## 3. Rendimiento y memoria

Compilación optimizada, 1440 × 900, dibujo por software (llvmpipe). Para no
confundir la máquina con el cambio, se midieron `main` (R4.5) y R5 alternados,
dos corridas cada uno, en esta misma sesión.

| Proyecto | Vista | Mediana `main` | Mediana R5 | p95 `main` | p95 R5 | Memoria máx. `main` | Memoria máx. R5 |
|---|---|---|---|---|---|---|---|
| casa | unifilar | 0.70 · 0.69 ms | 0.74 · 0.71 ms | 1.03 · 1.02 | 1.02 · 0.99 | 169 MiB | 161 · 165 MiB |
| casa | revisar | 0.55 · 0.57 ms | 0.59 · 0.55 ms | 0.77 · 0.81 | 0.78 · 0.69 | 165 · 167 MiB | 159 · 163 MiB |
| casa | memoria | 0.33 · 0.31 ms | 0.31 · 0.31 ms | 0.46 · 0.36 | 0.53 · 0.51 | **335 · 329 MiB** | **185 · 184 MiB** |
| grande | unifilar | 4.57 · 4.37 ms | **3.40 · 3.38 ms** | 6.69 · 6.53 | 4.75 · 4.64 | 188 MiB | 184 MiB |
| grande | revisar | 10.80 · 10.59 ms | 9.55 · 9.52 ms | 13.88 · 13.32 | 11.31 · 12.53 | 187 MiB | 183 MiB |
| grande | memoria | no termina | 0.61 · 0.61 ms | — | 0.80 · 0.79 | **más de 5,370 MiB (se detuvo)** | **553 · 554 MiB** |

- **Memoria con la casa:** de 330 a 185 MiB. **Con el grande (522 páginas):**
  antes no terminaba; ahora 553 MiB, de los que unos 385 son el documento
  compuesto (medido en R1).
- **El cuadro en que se compone** sigue siendo lento y bloquea la ventana: con
  la casa, 74 a 88 ms; con el grande, 1.26 a 1.31 s. Es la composición de
  Typst, que R5 no cambia; antes se sumaba además la conversión de todas las
  páginas. Después de componer, cada cuadro tarda menos de 1 ms.
- **Unifilar del grande:** 25 % menos por cuadro (mediana de 4.5 a 3.4 ms).
- **Revisar del grande:** 10 % menos (de 10.7 a 9.5 ms). La miniatura del
  unifilar ya no dibuja lo que queda fuera, pero **no era el costo principal**:
  lo que queda es la columna con el resumen de 20 tableros y la lista de
  pendientes, que se arman en cada cuadro. Esa vista se rehace en R7, cuando
  se retira `guided.rs`; se anota para entonces y para la etapa 5 del plan
  (rendimiento).
- El tope de seis imágenes y qué páginas se piden se prueban con pruebas
  automáticas (§5); la medición de la app es sin desplazarse.

## 4. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (5 min 38 s). El
módulo pesa 50,781,449 bytes contra 50,749,631 de R4.5: 31,818 bytes más
(0.06 %).

Se abrió el sitio en Chromium sin pantalla (Playwright, 1440 × 900), se cargó
cada proyecto y se entró a Memoria. Se midió la pausa más larga en que la
pestaña no respondía durante 10 s y la memoria de los procesos de Chromium.
En R1 se midió del clic a la primera respuesta, que con la vista de entonces
era lo mismo, porque todo pasaba en un solo cuadro.

| Proyecto | Pausa R1 | Pausa R5 | Memoria que agrega Chromium R1 | R5 |
|---|---|---|---|---|
| casa (13 págs.) | 1.4 s | 0.7 s | 175 MiB | 42 MiB |
| grande (522 págs.) | 13.3 s | 2.9 s | 5,215 MiB | 376 MiB |

La pausa que queda es la composición de Typst. En un teléfono real no se
probó; con 376 MiB en vez de 5.2 GB, ya no debería cerrarse la pestaña, pero
**no está medido**.

## 5. Pruebas

Nuevas:

- `the_composed_report_has_the_pages_of_the_pdf` (`ieestudio-reports`): la
  memoria compuesta tiene las mismas páginas que el PDF, de 612 × 792 puntos;
  una página se convierte al tamaño pedido (1× y 2×) y un número de página
  fuera del documento no devuelve nada.
- `only_the_pages_in_view_and_one_on_each_side_are_wanted`: con 522 páginas,
  al principio, a la mitad y al final del documento; alejado, varias a la vez.
- `at_most_six_images_are_kept_and_the_farthest_go_first`.
- `pages_are_stacked_with_a_gap`.
- `zooming_keeps_the_point_under_the_cursor_in_place`: el punto del dibujo bajo
  el cursor queda en el mismo lugar al acercar; al alejar no quedan
  desplazamientos negativos.
- `what_is_out_of_view_is_not_drawn`: un texto fuera de la vista no se dibuja;
  uno que empieza fuera pero entra, y una línea que la cruza, sí.

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para `wasm32`
y `cargo test` pasan: **296 pruebas, 0 fallas**. Ninguna prueba de cálculo
cambió. La prueba del sistema visual pasa con el código nuevo.

## 6. Capturas

En `docs/capturas-r5/`, cada imagen pone lado a lado la captura de R4.5
(`main`) y la de R5 en el mismo tamaño y tema:

- Memoria a 1440 × 900, oscuro y claro.
- Unifilar a 1440 × 900, oscuro.
- Memoria y unifilar a 390 × 844, claro.

Matriz completa de `tools/capturas.sh` (compilación optimizada), una vez al
final: **70 de 70 capturas sin fallas**, en los dos temas. Además, **28 de 28**
en compilación de depuración en los cuatro tamaños angostos, para que un ancho
negativo cierre el programa si lo hubiera.

## 7. Límites y lo que queda

- **Ningún gesto se probó con una mano real.** El pellizco, la rueda con Ctrl y
  el arrastre usan lo que egui entrega (`zoom_delta` y el arrastre del área
  desplazable); la cuenta que mantiene fijo el punto bajo el cursor tiene su
  prueba, pero ningún clic ni gesto se probó con pruebas automáticas de la
  interfaz. El pellizco en teléfono se valida en R6.
- La composición de la memoria del proyecto grande sigue bloqueando la ventana
  más de un segundo, en escritorio, y más en el navegador (3.3 s medidos en
  R1). Componer en otro hilo no es posible en el navegador sin cambiar cómo se
  compila; queda para la etapa 5 del plan.
- La vista Revisar (de `guided.rs`) solo ganó lo de la miniatura; su arreglo
  completo es de R7.
