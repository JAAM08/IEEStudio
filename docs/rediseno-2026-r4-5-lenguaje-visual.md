# Rediseño 2026 — R4.5: lenguaje visual

**Fecha:** 19 de noviembre de 2026.
**Base:** `main` con R4 integrado (PR #27).
**Referencias:** revisión «Rediseño 2026 — Revisión de la dirección visual
antes de R5» y lienzo de Claude Design aprobado con ajustes el 19 de
noviembre (mesas 5, 6 y 7).
**Sistema resultante:** `docs/sistema-visual.md`, obligatorio para R5, R6 y R7.

R4.5 cambia solo el acabado. No toca el motor, los cálculos, la lógica
eléctrica, las tablas, el formato `.iees`, la memoria en PDF ni el dibujo del
unifilar, y no agrega funciones. La paleta aprobada en R2 no cambia; se agrega
un divisor discreto (`hair`), un gris intermedio entre el panel y la línea de
borde.

## 1. Qué cambia para quien usa el programa

- **Barra de vistas en cápsula**, centrada en la barra superior. La vista
  activa va elevada con su ícono en amarillo; ya no lleva subrayado. Revisar
  muestra sus pendientes en una insignia.
- **Navegador integrado al cromo:** va en el mismo tono que la barra superior y
  la de estado. La selección es una píldora; ya no hay barra amarilla al lado.
- **Ficha agrupada por secciones:** Circuito, Carga, Conexión, Instalación,
  Interruptor y Ajustes manuales son grupos compactos con renglones de 28 px y
  divisores finos. Las pestañas Datos, Resultados y Normativa son un control en
  cápsula.
- **Cuadro de cargas:** directo sobre el contenido, sin barra amarilla en la
  fila elegida, con rótulos en tipo oración («Conectada», «Lo que capturas»).
  Las etiquetas de estado son cápsulas.
- **Vista Proyecto:** el resumen es un solo recuadro con las cuatro cifras
  separadas por divisores, y los bloques ya no van en recuadros propios: un
  divisor y su título.
- **Barra de estado discreta:** texto secundario en gris, pero conserva los tres
  niveles con su marca: cálculo (punto del color de su estado), cumplimiento
  (punto del color de su estado) y aprobación profesional pendiente (anillo
  vacío: la da el ingeniero, nunca el programa). «Siguiente» es un botón de
  texto; el amarillo queda para la acción principal de cada vista.
- **Lo que flota** (menús, ventanas, avisos) va en la superficie elevada con
  una sombra suave, la única del sistema.
- **Tipografía:** cinco tamaños (22, 15, 13, 12 y 11) más tres excepciones
  documentadas; antes había veinte. Ningún rótulo va en mayúsculas.

## 2. Lo que se fijó como sistema

| Tema | Antes (R4) | Ahora (R4.5) |
|---|---|---|
| Tamaños de letra | 20 distintos | 5 y 3 excepciones documentadas |
| Rótulos en mayúsculas | 29 | 0 |
| Márgenes y espacios | 4 a 40 px sin regla | 4, 8, 12, 16, 24 y 32 px (y 2 px dentro de cápsulas) |
| Superficies | Navegador, contenido e inspector en el mismo gris | Cromo, contenido y elevado; sombra solo en lo que flota |
| Indicadores de selección | Fondo y barra o subrayado amarillo | Uno solo: cápsula o píldora |
| Alturas de control | 22, 24, 26 y 30 px | 24 en fichas, 28 en barras y renglones (44 táctil en R6) |

Una prueba nueva (`the_interface_uses_only_the_visual_system`) revisa todo el
código de la interfaz y falla si aparece un tamaño de letra fuera de la escala,
un margen o espacio fuera de la retícula, o un rótulo convertido a mayúsculas.
Así R5, R6 y R7 no pueden salirse del sistema sin que se note.

## 3. Capturas antes y después

En `docs/capturas-r4-5/`, cada imagen pone lado a lado la captura de R4 y la de
R4.5 en el mismo tamaño y tema:

- Tableros y circuitos a 1440 × 900, oscuro y claro.
- Proyecto a 1440 × 900, oscuro y claro.
- Revisar a 1440 × 900, oscuro.
- Procedimiento abierto a 1440 × 900, claro.
- Tableros y circuitos a 390 × 844, oscuro, y Proyecto a 390 × 844, claro.

Matriz completa de `tools/capturas.sh` (compilación optimizada), una vez al
final: **70 de 70 capturas sin fallas**, en los dos temas. Además, **28 de 28**
en compilación de depuración en los cuatro tamaños angostos.

## 4. Pruebas

- Nueva: el código de la interfaz solo usa la escala tipográfica, la retícula y
  rótulos sin mayúsculas.
- Las de contraste de R2 siguen pasando con el divisor nuevo.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para
  `wasm32` y `cargo test` pasan (291 pruebas, 0 fallas). Ninguna prueba de
  cálculo cambió.

## 5. Rendimiento

Compilación optimizada, 1440 × 900. La primera medición comparada con la tabla
de R4 salió entre 30 y 50 % más lenta **en todas las vistas por igual**, más
que el ruido de 15 a 25 % medido en R1. Para saber si era R4.5 o la máquina, se
compilaron R4 (`main`) y R4.5 en esta misma sesión y se midieron alternados,
dos corridas cada uno:

| Proyecto | Vista | Mediana R4 (dos corridas) | Mediana R4.5 (dos corridas) | p95 R4 | p95 R4.5 |
|---|---|---|---|---|---|
| casa | proyecto | 0.45 · 0.48 | 0.60 · 0.46 | 0.66 · 0.86 | 1.03 · 0.89 |
| casa | cuadro | 1.44 · 1.38 | 1.47 · 1.57 | 2.42 · 2.46 | 2.36 · 2.53 |
| casa | unifilar | 0.99 · 0.81 | 1.07 · 1.06 | 1.35 · 1.51 | 1.55 · 1.47 |
| casa | revisar | 0.78 · 0.87 | 0.79 · 0.76 | 1.14 · 1.48 | 1.25 · 1.29 |
| casa | procedimiento | 1.59 · 1.89 | 1.62 · 1.91 | 2.38 · 3.03 | 2.15 · 2.68 |
| casa | dividida | 1.43 · 2.04 | 1.68 · 1.62 | 2.40 · 3.10 | 2.66 · 2.80 |
| grande | proyecto | 0.92 · 1.16 | 1.06 · 1.27 | 1.27 · 1.72 | 1.44 · 2.03 |
| grande | cuadro | 4.74 · 5.79 | 4.97 · 6.72 | 6.70 · 8.50 | 7.36 · 8.83 |
| grande | unifilar | 6.62 · 5.51 | 6.09 · 6.58 | 9.36 · 8.33 | 8.38 · 8.75 |
| grande | revisar | 17.20 · 13.48 | 13.75 · 12.39 | 21.32 · 19.92 | 18.34 · 17.17 |
| grande | procedimiento | 6.64 · 5.65 | 5.90 · 4.95 | 8.01 · 9.24 | 7.93 · 6.91 |
| grande | dividida | 5.59 · 5.42 | 4.64 · 4.39 | 8.10 · 6.60 | 6.46 · 6.64 |

Las dos versiones quedan en el mismo rango: la diferencia era de la máquina de
hoy, no del sistema visual. El caso más pesado sigue siendo Revisar con 386
circuitos (mediana de 12 a 17 ms en las dos versiones, con un máximo de 32.7 ms
en una corrida de R4.5); ya estaba anotado desde R1 y su corrección (no dibujar
la miniatura del unifilar en cada cuadro) es de R5. La memoria no cambia
(162 a 189 MiB, igual que en R4).

## 6. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (6 min 58 s). El
módulo pesa 50,749,631 bytes contra 50,552,735 de R4: 196,896 bytes más
(0.4 %). No se midió qué parte del código nuevo los aporta. Solo se verificó que compila y se arma; no se abrió en un
navegador.

## 7. Límites y lo que queda

- Las pantallas de teléfono (barra inferior, hojas, renglones de 44 px) y la
  pantalla de inicio con dos experiencias son de R6; el sistema ya tiene sus
  medidas, pero todavía no se usan.
- La vista Memoria y el unifilar reciben su acabado en R5 (controles flotantes
  en cápsula y ficha de Memoria en grupos).
- `guided.rs` (contenido de ventanas angostas, Revisar y Memoria) usa ya la
  escala y la retícula; se retira en R7.
- La revisión visual detallada se hizo sobre capturas muestreadas y las
  comparativas; el resto de la matriz se verificó automáticamente.
- Ningún clic se probó con pruebas automáticas de la interfaz.
