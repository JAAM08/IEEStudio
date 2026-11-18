# Rediseño 2026 — R3: marco único en escritorio

**Fecha:** 18 de noviembre de 2026.
**Base:** `main` con R2 integrado (PR #25).
**Referencias:** maqueta R0 aprobada (secciones 4 a 6) y sistema visual de R2.

R3 cambia el marco de la aplicación: desaparecen el modo guiado y el modo
avanzado y queda una sola aplicación. No toca el motor, las tablas, las
decisiones técnicas, el formato `.iees`, la memoria en PDF ni el dibujo del
unifilar. El contenido de cada pantalla es el que ya existía; unificarlo es
trabajo de R4 y R5.

## 1. Qué cambia para quien usa el programa

- **Una sola barra superior** con el menú (≡: Archivo, Edición, Ver,
  Herramientas y Ayuda), el nombre del proyecto, deshacer y rehacer, las cinco
  vistas y, a la derecha, cálculo rápido, dos vistas, procedimiento, búsqueda y
  tema.
- **Cinco vistas**, en el orden de la ruta de trabajo: Proyecto, Tableros y
  circuitos, Unifilar, Revisar y Memoria. La vista activa lleva una línea
  amarilla. Revisar muestra cuántos puntos faltan por resolver, desde cualquier
  vista.
- **Regiones fijas** en las vistas de trabajo (Tableros y circuitos, Unifilar):
  el navegador del proyecto a la izquierda, la ficha del elemento elegido
  (Datos, Resultados, Normativa) a la derecha. Elegir algo en cualquiera lo
  marca en las demás.
- **Procedimiento** en un panel inferior que se abre desde la barra, el menú
  Ver, la barra contextual del cuadro o con F7.
- **Cálculo rápido** como ventana flotante, desde la barra o con F8.
- **«Siguiente: …»** en la barra de estado ofrece la vista que sigue en la ruta,
  sin obligar el orden.
- **Dos vistas a la vez** (cuadro y unifilar lado a lado) en ventanas de 1360 px
  o más; el navegador se contrae mientras están abiertas.
- **Teclas:** F2 a F6 van a cada vista, F7 abre o cierra el procedimiento y F8
  el cálculo rápido. Antes F2 a F8 abrían y cerraban paneles.
- **Se retira:** el cambio entre vista guiada y avanzada, los espacios de
  trabajo (Cálculo, Diagrama, Documentación), arrastrar y soltar paneles, los
  botones «Anterior» y «Siguiente» del pie de cada paso y la barra de pasos
  numerada.
- **Preferencias de antes:** lo que se guardó del modo, el acomodo de paneles y
  el paso se ignora sin error; la primera vez se abre en Tableros y circuitos.

## 2. Ventanas angostas

R3 es el marco de escritorio. Con menos de 900 px no hay regiones laterales:
cada vista se muestra sola, y «Tableros y circuitos» conserva sus dos
contenidos de antes (Tableros y Circuitos, con su ficha) elegibles arriba. Los
cajones, las hojas y la barra inferior son de R6; mientras tanto nada de lo que
funcionaba a 390 o 600 px dejó de funcionar.

## 3. Funciones de la sección 6 de la maqueta que preserva esta etapa

| Fila | Función | Dónde queda | Cómo se comprobó |
|---|---|---|---|
| 1 | Pantalla de inicio | Sin cambios; se abre al arrancar sin proyecto o con el comando «Inicio» de la paleta | Captura `--inicio` sin cierres |
| 18 | Procedimiento | Panel inferior; en ventanas angostas, también abajo (la pantalla completa es de R6) | Capturas con el panel abierto en 10 combinaciones |
| 21, 22 | Revisar y sus tarjetas «Por resolver» | Vista Revisar, mismo contenido; «Abrir unifilar» ahora cambia a la vista Unifilar | Capturas; prueba de qué vista corresponde a cada paso |
| 25 | Exportar PDF, Excel, DXF y SVG | ≡ › Archivo y botones de su vista | Sin cambio de código en los menús |
| 27 | Cálculo rápido flotante | Ventana flotante, F8 | Captura `--rapido` |
| 28 | Paleta de comandos | Igual; ya lista las cinco vistas y no los modos | Prueba de la paleta; captura `--comandos` |
| 29 | Deshacer y rehacer | Barra superior y ≡ › Edición | Captura |
| 30 | Tema claro y oscuro | Barra superior y ≡ › Ver | Capturas en ambos temas |
| 31 | Color de acento | Ya retirado en R2 | — |
| 32 | Paneles acoplables | Regiones fijas y dos vistas a la vez; `egui_dock` ya no se usa | Captura de dos vistas |
| 33 | Insignia de pendientes y «Siguiente» | Barra de vistas y barra de estado | Capturas |
| 34 | Opciones de captura | `--vista`, `--procedimiento`, `--dividir` nuevas; `--paso`, `--espacio`, `--panel`, `--avanzada` se aceptan hasta R7 | `tools/capturas.sh` |
| 35 | Archivos por `files` | Sin cambio | — |

## 4. Pruebas

- `layout.rs`: qué regiones caben según el ancho (1440, 1024, 900 y menos de
  900) y que el centro nunca baja del mínimo, incluso con dos vistas.
- `views.rs`: orden de la ruta, atajos y títulos distintos, qué vistas tienen
  regiones, vista de omisión.
- `shell.rs`: a qué vista lleva cada paso que pide una vista.
- `commands.rs`: la paleta llega a cada vista con su atajo y no ofrece modos ni
  espacios de trabajo.
- `app.rs`: las preferencias guardadas por versiones anteriores se leen sin
  error, y la vista y la división se recuerdan.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para
  `wasm32` y `cargo test` pasan.

## 5. Capturas

Matriz completa de `tools/capturas.sh` (compilación optimizada), ejecutada una
vez al final: **70 de 70 capturas sin fallas** (siete vistas × cinco tamaños ×
dos temas; «procedimiento» reemplaza a la vieja «avanzada»). Además, 28 capturas
en compilación de depuración en los cuatro tamaños angostos, que es la que se
cierra con un ancho negativo: **28 de 28 sin fallas**. Capturas representativas
en `docs/capturas-r3/`.

## 6. Rendimiento

Compilación optimizada, 1440 × 900, misma máquina que R1 y R2. «R2» es la
medición de R2; donde la vista no existía, se compara el cuadro con paneles de
R2 («avanzada»).

| Proyecto | Vista | Mediana R2 → R3 (ms) | p95 R2 → R3 (ms) | Máx. R3 (ms) | Memoria pico R2 → R3 (MiB) |
|---|---|---|---|---|---|
| casa | proyecto | 0.24 → 0.29 | 0.36 → 0.42 | 0.47 | 166 → 163 |
| casa | cuadro | 1.11 → 1.06 | 1.73 → 1.53 | 1.74 | 170 → 166 |
| casa | unifilar | nueva → 0.62 | nueva → 0.83 | 0.96 | — → 168 |
| casa | revisar | 0.42 → 0.53 | 0.71 → 0.73 | 0.91 | 165 → 166 |
| casa | procedimiento | 1.11 → 1.18 | 1.73 → 1.49 | 1.53 | 170 → 166 |
| casa | dividida | nueva → 1.10 | nueva → 1.61 | 1.73 | — → 167 |
| taller | proyecto | 0.29 → 0.30 | 0.47 → 0.40 | 0.54 | 166 → 163 |
| taller | cuadro | 1.23 → 1.10 | 2.12 → 1.37 | 2.33 | 170 → 166 |
| taller | unifilar | nueva → 0.68 | nueva → 1.20 | 1.42 | — → 166 |
| taller | revisar | 0.56 → 0.73 | 0.80 → 0.93 | 1.12 | 170 → 168 |
| taller | procedimiento | 1.23 → 1.21 | 2.12 → 1.80 | 2.08 | 170 → 163 |
| taller | dividida | nueva → 1.10 | nueva → 1.31 | 1.67 | — → 168 |
| grande | proyecto | 0.50 → 0.55 | 0.62 → 0.75 | 0.85 | 174 → 172 |
| grande | cuadro | 3.11 → 3.86 | 4.58 → 5.31 | 6.43 | 179 → 178 |
| grande | unifilar | nueva → 4.32 | nueva → 5.86 | 6.43 | — → 188 |
| grande | revisar | 9.47 → 10.20 | 13.63 → 12.61 | 15.96 | 189 → 187 |
| grande | procedimiento | 3.11 → 3.63 | 4.58 → 5.42 | 5.72 | 179 → 177 |
| grande | dividida | nueva → 3.53 | nueva → 4.05 | 4.79 | — → 180 |

Las medianas quedan dentro del ruido de 15 a 25 % medido en R1. El caso más
pesado (Grande, Revisar) tiene mediana de 10.2 ms y p95 de 12.6 ms; su cuadro
más lento fue de 15.96 ms, justo bajo los 16 ms de un cuadro a 60 Hz.

## 7. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (5 min 25 s). El
módulo pesa 50,472,099 bytes (≈ 48.1 MiB) contra 50,778,609 de R2: **306,510
bytes menos**, porque la aplicación dejó de usar el código de los paneles
acoplables. Solo se verificó que compila y se arma; no se abrió en un navegador.

## 8. Límites y lo que queda

- `guided.rs` sigue dando el contenido de Proyecto, Revisar y Memoria, y el de
  Tableros y circuitos en ventanas angostas; se retira en R7 junto con
  `egui_dock`, que ya no se usa pero sigue declarado en `Cargo.toml` (decisión
  D-3).
- En ventanas de 1360 a 1500 px con dos vistas a la vez, el cuadro queda
  angosto y su barra de acciones se apretuja; el cuadro unificado de R4 lo
  resuelve. El unifilar de esa vista no se ajusta solo al ancho: se usa
  «Ajustar al ancho».
- Los contadores que filtran Revisar, el cuadro con juegos de columnas y la
  ficha única son de R4; el unifilar táctil y la memoria por páginas visibles,
  de R5; cajones, hojas y barra inferior, de R6.
- Las mediciones son de una máquina con dibujo por software; sirven para
  comparar, no como promesa para otra computadora.
- Ningún clic se probó con pruebas automáticas de la interfaz (no existen en
  este proyecto); la revisión de comportamiento fue con capturas.
