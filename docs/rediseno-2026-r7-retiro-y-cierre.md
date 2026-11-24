# Rediseño 2026 — R7: retiro y cierre

**Fecha:** 24 de noviembre de 2026.
**Base:** `main` con R6 integrado (PR #30).
**Decisión:** `decisions/0034-marco-unico-adaptativo.md`, que sustituye a la
0010 y a la 0020.

R7 termina el rediseño: saca Revisar de la vista guiada y la rehace, retira
lo que quedaba de las interfaces de antes, hace el repaso final contra el
sistema visual, actualiza el sitio y deja la tabla final de rendimiento. No
toca el motor, los cálculos, las tablas, el formato `.iees`, el contenido de
la memoria en PDF ni el dibujo del unifilar.

## 1. Revisar, rehecha

Revisar vive ahora en `panels/review.rs`, con el mismo contenido y los mismos
arreglos que antes: conexiones que el sistema no tiene, topología sin
declarar, tipo de carga dudoso, transformador repetido, cada incidencia con su
detalle y su artículo, capacidad interruptiva por lugar, fuera de alcance y
avisos.

- **Escritorio:** el dictamen y la lista al centro; **la ficha del elemento
  elegido a la derecha**. «Ver ficha» en una incidencia la muestra ahí sin
  salir de Revisar, y la incidencia elegida se marca en amarillo.
- **Tableta:** la ficha sale en su cajón.
- **Teléfono:** el dictamen, **contadores en dos columnas** (No cumplen, Sin
  evaluar, Tramos calculados, Fuera de alcance) y las incidencias como
  tarjetas; «Abrir ficha» lleva a la ficha a pantalla completa en Circuitos.
- **Contadores** también en escritorio y tableta, en un renglón.
- **Menos amarillo:** «Usar 10 AWG» ya no es amarillo en cada tarjeta; el
  amarillo queda para la acción principal de cada vista.
- **Sin miniatura del unifilar:** el unifilar tiene su vista. El resumen por
  tablero (carga, corriente, caída máxima y falla) sigue, plegado al final,
  en grupos de campos.
- **Más rápida con proyectos grandes:** las incidencias se arman una vez por
  cambio del proyecto, no en cada cuadro, y se muestran de 40 en 40 con
  «Mostrar 40 más».

## 2. Lo que se retiró

| Qué | Dónde quedó lo que hacía |
|---|---|
| `guided.rs` (2,339 líneas): pasos de la vista guiada, tabla y tarjetas de circuito angostas, unifilar con barra | Revisar en `panels/review.rs`; el unifilar de escritorio en `panels/diagram.rs`; lo demás ya lo hacían las vistas del marco desde R3 a R6 |
| La dependencia `egui_dock` | Nada: no se usaba desde R3 |
| Opciones `--paso`, `--espacio`, `--panel`, `--avanzada`, `--unifilar` y `--unifilar-completo` | `--vista`, `--ficha` y `--rapido` |
| El aviso de «pantallas grandes» de la versión web | El marco de teléfono de R6 |
| `Step` y `shell::target` | Las vistas piden ir a otra vista directamente |

Las preferencias guardadas por versiones anteriores (`guided`, `workspace`)
siguen cargando: lo que ya no existe se ignora (prueba
`preferences_from_before_the_single_frame_still_load`).

## 3. Repaso final contra el sistema visual

- La prueba `the_interface_uses_only_the_visual_system` ahora también revisa
  los tamaños escritos como `FontId::monospace(…)` y `FontId::proportional(…)`
  y los radios (`corner_radius`): solo 0, 6, 8 o 10 px.
- Se corrigieron los dos tamaños fuera de escala que quedaban (10.5 y 11 px en
  «¿Qué vas a conectar?», ahora `FOOT`) y los radios escritos como número
  pasaron a sus nombres (`RADIUS_CARD`, `RADIUS_WINDOW`).
- Con `guided.rs` se fueron los últimos rótulos con estilo de antes.

## 4. Sitio

- Capturas nuevas de la portada (`app-calculo`, `app-unifilar`, `app-rapido`)
  y de los diez pasos de la guía, con la zona de cada paso resaltada.
- La guía describe la interfaz nueva: la pantalla de inicio con dos
  experiencias, las cinco vistas, la ficha, Revisar, los atajos F2 a F8 y el
  teléfono. El texto se cambió en `website/guia/index.html` y en
  `tools/paginas.py`.

## 5. Pruebas

- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para
  `wasm32` y `cargo test` pasan: **304 pruebas, 0 fallas**. Ninguna prueba de
  cálculo cambió.
- Las cuatro pruebas de Revisar pasaron con su código a `panels/review.rs`
  (lo que cuenta la barra es lo que lista Revisar; primero lo que no cumple,
  con su arreglo; solo se pregunta el tipo de las cargas generales; cifras con
  separador de miles).
- Nuevas: `incidents_are_built_once_per_change`,
  `picking_an_incident_depends_on_the_size` y
  `every_run_of_the_example_is_calculated`.
- Se quitaron las pruebas de lo retirado (`shell::target`,
  `layout::side_fits`).
- La prueba del sistema visual revisa más casos (sección 3).

## 6. Capturas

- `docs/capturas-r7/`: Revisar antes (R6) y después (R7) a 1440 × 900,
  768 × 1024 y 390 × 844.
- Matriz completa de `tools/capturas.sh` (compilación optimizada): **100 de
  100 sin fallas**, diez vistas, cinco tamaños, dos temas. En compilación de
  depuración, **40 de 40** en los cuatro tamaños angostos.
- Validación táctil en la versión web (Chromium, Pixel 7 emulado, gestos
  táctiles reales), además de la de R6: Revisar se desliza; «Abrir ficha» en
  una incidencia lleva a la ficha a pantalla completa; «‹ TG-1» vuelve a las
  tarjetas; al volver a Revisar se conserva el desplazamiento y la incidencia
  elegida queda marcada. Sin errores en la consola.

## 7. Rendimiento: tabla final

Compilación optimizada, 1440 × 900, dibujo por software (llvmpipe). R6
(`main`) y R7 se midieron alternados, dos corridas cada uno, en esta sesión.
Mediana del tiempo por cuadro en milisegundos y memoria máxima del proceso:

| Proyecto | Vista | R6 | R7 | Memoria R6 | Memoria R7 |
|---|---|---|---|---|---|
| casa | proyecto | 0.41 · 0.41 | 0.41 · 0.42 | 160 MiB | 161 MiB |
| casa | cuadro | 1.15 · 1.15 | 1.17 · 1.20 | 164 MiB | 164 MiB |
| casa | unifilar | 0.76 · 0.77 | 0.76 · 0.78 | 166 MiB | 166 MiB |
| casa | revisar | 0.62 · 0.63 | 0.82 · 0.83 | 164 MiB | 164 MiB |
| casa | memoria | 0.34 · 0.33 | 0.35 · 0.34 | 186 MiB | 186 MiB |
| taller | revisar | 0.82 · 0.78 | 1.01 · 1.02 | 166 MiB | 163 MiB |
| taller | memoria | 0.34 · 0.33 | 0.34 · 0.32 | 197 MiB | 197 MiB |
| grande | proyecto | 0.86 · 0.89 | 0.90 · 0.90 | 171 MiB | 171 MiB |
| grande | cuadro | 3.96 · 3.98 | 4.06 · 4.00 | 176 MiB | 175 MiB |
| grande | unifilar | 3.48 · 3.79 | 3.57 · 3.49 | 185 MiB | 184 MiB |
| grande | revisar | 9.70 · 10.36 | **7.89 · 7.72** | 184 MiB | 174 MiB |
| grande | procedimiento | 3.92 · 3.98 | 4.00 · 4.21 | 175 MiB | 174 MiB |
| grande | dividida | 2.28 · 2.30 | 2.21 · 2.19 | 176 MiB | 176 MiB |
| grande | memoria | 0.63 · 0.67 | 0.62 · 0.61 | 553 MiB | 554 MiB |

- Revisar con 386 circuitos baja de unos 10 ms a 7.8 ms (−22 %) y 10 MiB
  menos. Con la casa y el taller sube unos 0.2 ms por los contadores y la
  ficha al lado; sigue por debajo de 1.1 ms.
- Lo demás queda igual que en R6, dentro del ruido.
- En el teléfono (390 × 844), Revisar del proyecto grande: mediana 7.46 ms,
  p95 9.90 ms.

**Contra la línea base de R1** (17 de noviembre, otra corrida de la misma
máquina; desde el 19 de noviembre la máquina mide entre 30 y 50 % más lento,
así que la comparación es indicativa):

| Medida | R1 | R7 |
|---|---|---|
| Memoria con el proyecto de 522 páginas | más de 5.4 GB (la prueba se detenía) | 554 MiB |
| Memoria con la casa en Memoria | 335 MiB | 186 MiB |
| Memoria con el taller en Memoria | 451 MiB | 197 MiB |
| Revisar con 386 circuitos, mediana | 12.84 ms | 7.8 ms |
| Unifilar con 386 circuitos, mediana | sin medir en R1 (4.5 ms en R4.5) | 3.5 ms |
| Versión web, memoria de 522 páginas | 13.3 s congelada, 5.2 GB | 2.9 s, 376 MiB (R5) |
| Peor cuadro de escritorio con 386 circuitos | Revisar, 15.5 ms de p95 | Revisar, 10 ms de p95 |

La meta del plan, **un cuadro por debajo de 16 ms con el proyecto de 386
circuitos**, se cumple en todas las vistas. La de **imágenes de la memoria por
debajo de 64 MB** también: como mucho seis páginas, menos de 45 MiB aun a
la nitidez máxima (2 píxeles por punto). Lo que
sigue pendiente es el cuadro en que se compone la memoria (1.3 s en
escritorio con 522 páginas), anotado para la etapa 5 del plan.

## 8. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (4 min 36 s). El
módulo pesa 50,765,793 bytes contra 50,909,200 de R6: **143,407 bytes menos**,
por lo retirado.

## 9. Para probar en tu teléfono

El plan ponía tu revisión en un teléfono real antes de R7. Sigue pendiente;
esto es lo que conviene probar en la versión web (y en iOS Safari si tienes
un iPhone a la mano):

1. Abrir el sitio y entrar a la app: debe verse la barra inferior con los
   cinco destinos, sin el aviso de «pantallas grandes».
2. Circuitos: tocar una tarjeta, deslizar la ficha, cambiar un dato con el
   teclado en pantalla y volver con «‹ TG-1».
3. Unifilar: pellizcar para acercar, arrastrar, tocar un circuito y «Abrir
   ficha».
4. Revisar y Memoria: deslizar hasta el final.
5. Menú → Cálculo rápido → «‹ Inicio».
6. Girar el teléfono: en horizontal pasa al marco de tableta o de escritorio,
   según el ancho que quede.

## 10. Límites

- No se probó en un teléfono ni una tableta reales.
- Componer la memoria sigue bloqueando la ventana con proyectos grandes.
- Las incidencias de Revisar se muestran de 40 en 40; con muchas, hay que
  pulsar «Mostrar 40 más».
- La guía del sitio tiene capturas nuevas, pero el video de la guía
  (`website/video/guia.mp4`) es de antes del rediseño; rehacerlo es otro
  trabajo.
- Las capturas nuevas del sitio miden 1440 × 900; las de la portada eran de
  2400 × 1500, así que en pantallas de alta densidad se ven menos nítidas.
