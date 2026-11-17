# Rediseño 2026 — R1: línea base, Revisar en ventanas angostas y mediciones

**Fecha:** 17 de noviembre de 2026.
**Base:** `main` en `74aa56e` (merge del PR #23), sin cambios pendientes ni PR
abiertos. Es la misma base de la maqueta R0.
**Referencias:** diagnóstico del rediseño y maqueta R0 aprobada (documentos
del proyecto en claude.ai).

R1 no cambia el motor, las tablas, las decisiones técnicas, el formato
`.iees`, la memoria en PDF ni el unifilar. Corrige el cierre de Revisar en
ventanas angostas, la actualización de la versión web y deja medidos los dos
riesgos que la maqueta R0 dejó abiertos: el teclado del teléfono y la memoria
de la vista previa.

Todas las mediciones son de una máquina Linux con 2 núcleos Xeon a 2.1 GHz,
7 GB de memoria y dibujo por software (llvmpipe), con compilación optimizada.
Los números absolutos sirven para comparar antes y después en la misma
máquina, no como promesa para otra computadora.

## 1. Cómo se ve cada vista antes de R1

Capturas con `tools/capturas.sh` (casa de ejemplo, tema oscuro y claro).

| Vista | 1440 × 900 | 1024 × 768 | 768 × 1024 | 600 × 844 | 390 × 844 |
|---|---|---|---|---|---|
| Proyecto | Correcta | Correcta | Correcta | Recortada a la derecha | Recortada; los campos en dos columnas no caben |
| Tableros | Correcta | Correcta | El árbol queda angosto junto a la ficha de 420 px | El título va una palabra por renglón; el árbol queda tapado | El árbol desaparece detrás de la ficha |
| Circuitos | Correcta | Correcta | Tabla apretada junto a la ficha de 390 px | La tabla queda tapada por la ficha | La tabla desaparece detrás de la ficha |
| Revisar | Correcta | Correcta | Correcta | **Se cierra** (compilación de depuración); en la optimizada, el texto va una letra por renglón encima del unifilar | **Se cierra**; en la optimizada, título, tarjetas y unifilar encimados |
| Memoria | Correcta | Correcta | Vista previa angosta | Vista previa cortada | Vista previa fuera de la pantalla |
| Unifilar completo | Correcto | Correcto | Correcto | Correcto | Correcto, con zoom |
| Vista avanzada | Correcta | Correcta | Paneles apretados | Paneles apretados | Paneles encimados |
| Barra superior (vista guiada) | Correcta | Correcta | Pasos debajo de «Cálculo rápido» | Encimados | Pasos y botones fuera de la pantalla |
| Barra de estado (todas) | Correcta | Correcta | Ruta y veredicto encimados | Encimados | Encimados |

Resultado de la matriz (5 tamaños × 7 vistas × 2 temas, compilación de
depuración, que revisa los anchos negativos): **66 de 70 sin fallas; 4 cierres**,
todos en Revisar a 600 y 390 px, con «Negative width makes no sense» (−10 y
−37.2 px). Con el taller de 22 páginas también se cerraba a 390 px.

**Causa.** Revisar ponía el unifilar en un panel fijo de 440 px y repartía lo
que sobraba entre tres tarjetas con una resta sin límite. Con 600 px de
ventana quedaban 96 px para las tres, y cada una pedía (96 − 24) / 3 − 4 − 30
= −10 px. Tableros, Circuitos y Memoria tienen el mismo patrón (paneles de
420, 390 y 440 px): no se cerraban, pero tapaban el contenido.

## 2. Qué cambió en la interfaz

- **Paneles de los pasos.** Si el panel lateral de un paso no cabe junto al
  contenido (menos de 420 px para el contenido), pasa abajo y a lo ancho, con
  su alto ajustable arrastrando el borde; en Memoria pasa arriba, porque son
  los datos de firma. Nada se quita: el unifilar en miniatura, el resumen por
  tablero, las fichas y los datos de firma siguen ahí. Con las medidas
  actuales, a 1024 px o más todo queda como antes; a 768 px o menos, apilado.
- **Tarjetas del veredicto en Revisar.** Tres en una fila mientras cada una
  mida al menos 150 px; si no, una debajo de otra.
- **Anchos nunca negativos.** Todas las restas de ancho de la vista guiada
  pasan por `layout.rs`, que nunca devuelve menos de cero.
- **Barra de estado.** Abajo de 900 px de ventana el veredicto va en un
  segundo renglón y se lee en el mismo orden (cálculo · verificación ·
  aprobación).
- **Barra superior de la vista guiada.** Abajo de 900 px la barra entera se
  recorre de lado (arrastrando o con la rueda): los pasos ya no quedan debajo
  de «Cálculo rápido» y los botones de la derecha ya no quedan fuera de la
  pantalla. Entre 900 y unos 960 px, si los pasos no caben, solo ellos se
  recorren de lado.
- **Escritorio sin cambios.** A 1440 × 900 las 14 capturas son idénticas píxel
  por píxel antes y después. A 1024 × 768 solo cambian franjas de 2 px de las
  barras de desplazamiento y 1,116 píxeles de suavizado en la página de la
  memoria; nada visible.

Después de R1: **70 de 70 capturas sin fallas** con la casa, **40 de 40** con el
taller (Revisar, Memoria, Tableros, Circuitos) y **9 de 9** con el proyecto
de 386 circuitos (Revisar, Tableros, Circuitos a 1024, 600 y 390 px).

Lo que R1 deja como está, por ser de etapas posteriores: Proyecto sigue en
dos columnas angostas a 390 px, el árbol de Tableros se recorta a la derecha
y la vista avanzada no se tocó (se retira en R7). Las barras que se recorren
de lado son una solución de paso; la navegación de teléfono (barra inferior y
hojas) es de R6.

## 3. Rendimiento de la interfaz

`ieestudio-app --captura … --medir 60` mide cuánto tarda la interfaz en armar
cada cuadro (sin contar el dibujo en la tarjeta de video) y la memoria
residente máxima del proceso. 1440 × 900, tema oscuro.

| Proyecto | Vista | Primer cuadro antes / después | Mediana antes / después | p95 antes / después | Memoria máx. antes / después |
|---|---|---|---|---|---|
| Casa | proyecto | 6.7 / 5.8 ms | 0.36 / 0.36 ms | 0.49 / 0.54 ms | 160 / 164 MiB |
| Casa | tableros | 10.5 / 6.9 ms | 0.61 / 0.52 ms | 1.01 / 0.70 ms | 161 / 163 MiB |
| Casa | circuitos | 7.2 / 6.8 ms | 0.84 / 0.83 ms | 1.08 / 1.35 ms | 160 / 163 MiB |
| Casa | revisar | 9.7 / 14.9 ms | 0.79 / 0.65 ms | 1.16 / 0.97 ms | 163 / 167 MiB |
| Casa | memoria | 409.0 / 289.9 ms | 0.37 / 0.39 ms | 0.66 / 0.55 ms | 331 / 335 MiB |
| Casa | avanzada | 19.0 / 13.8 ms | 1.90 / 1.46 ms | 2.26 / 2.39 ms | 164 / 165 MiB |
| Taller | proyecto | 5.2 / 6.0 ms | 0.50 / 0.52 ms | 0.72 / 0.71 ms | 161 / 164 MiB |
| Taller | tableros | 13.5 / 10.0 ms | 0.60 / 0.62 ms | 0.85 / 0.86 ms | 162 / 165 MiB |
| Taller | circuitos | 8.8 / 9.0 ms | 0.76 / 1.08 ms | 1.15 / 1.24 ms | 161 / 164 MiB |
| Taller | revisar | 10.5 / 14.7 ms | 0.89 / 0.89 ms | 1.36 / 1.37 ms | 165 / 168 MiB |
| Taller | memoria | 410.5 / 440.6 ms | 0.43 / 0.37 ms | 0.58 / 0.58 ms | 452 / 451 MiB |
| Taller | avanzada | 17.9 / 16.8 ms | 1.69 / 2.01 ms | 2.60 / 2.54 ms | 164 / 168 MiB |
| Grande | proyecto | 5.3 / 7.2 ms | 0.65 / 0.65 ms | 0.86 / 0.88 ms | 169 / 172 MiB |
| Grande | tableros | 7.9 / 7.2 ms | 1.02 / 1.04 ms | 1.38 / 1.53 ms | 170 / 173 MiB |
| Grande | circuitos | 7.4 / 11.1 ms | 1.24 / 1.40 ms | 1.79 / 2.09 ms | 170 / 169 MiB |
| Grande | revisar | 40.6 / 37.5 ms | 11.61 / 12.84 ms | 15.45 / 15.54 ms | 186 / 189 MiB |
| Grande | memoria | — | — | — | más de 5.4 GB en los dos (se detuvo la prueba) |
| Grande | avanzada | 16.7 / 19.0 ms | 3.97 / 4.75 ms | 5.34 / 6.33 ms | 174 / 175 MiB |

«Antes» es `main` con solo la opción `--medir` agregada; las dos columnas se
midieron una tras otra. La casa tiene 6 circuitos; el taller, 9 y 22 páginas
de memoria; el grande, 20 tableros, 386 circuitos y 522 páginas.

Entre corridas repetidas del mismo programa la mediana varía de 15 a 25 %
(Revisar con el grande: de 11.3 a 13.9 ms en seis corridas, tres de cada
versión) y el primer cuadro varía más. Las diferencias de la tabla están
dentro de ese ruido. La memoria sube unos 3 MiB, de forma constante. R1 no
cambia el tiempo por cuadro de forma medible.

Dos hallazgos para etapas siguientes:

- **Revisar con 386 circuitos tarda de 11 a 14 ms por cuadro**, contra 1 a 5 ms
  en las demás vistas. Lo más probable, por el código, es que sea la
  miniatura del unifilar completo, que se dibuja en cada cuadro; no se midió
  por partes. Va en R5 (unifilar) o antes, si se nota.
- **Memoria con 522 páginas no termina en esta máquina**: pasó de 5.4 GB y se
  detuvo la prueba para no agotar los 7 GB. Ver la sección 4.

## 4. Memoria técnica: composición y vista previa

### Cómo funciona hoy

Al entrar a Memoria, `report.rs` pide todas las páginas como imágenes a 1.6
píxeles por punto (979 × 1267 px, 4.73 MiB cada una) y las sube todas como
texturas. Durante la subida conviven tres copias: las imágenes que devuelve
`ieestudio-reports`, la copia que hace egui para subirlas y las texturas.

### Mediciones en escritorio

| Medida | Casa (13 págs.) | Taller (22 págs.) | Grande (522 págs.) |
|---|---|---|---|
| Calcular y escribir el texto | 1.8 ms | 2.4 ms | 41 ms |
| Componer (Typst) | 74 ms | 113 ms | 1,269 ms |
| Componer otra vez el mismo texto | 78 ms | 108 ms | 1,212 ms |
| PDF de lo ya compuesto | 13 ms (184 kB) | 28 ms (328 kB) | 505 ms (7.0 MB) |
| Una página a imagen: promedio / p95 / máximo | 8.2 / 18.6 / 18.6 ms | 8.5 / 10.8 / 23.8 ms | 5.1 / 10.7 / 73.8 ms |
| Todas las páginas a imagen | ≈ 0.1 s | ≈ 0.2 s | ≈ 2.7 s |
| Actualizar tras cambiar una longitud (calcular + componer + 1.ª página) | 1.5 + 68 + 8 ms | 2.1 + 120 + 8 ms | 55 + 1,247 + 19 ms |
| Memoria que agrega el documento compuesto | 26 MiB | 38 MiB | 385 MiB |
| Primer cuadro al entrar a Memoria (app) | 254 ms | 392 ms | no termina |
| Memoria máxima de la app en Memoria | 320 MiB | 455 MiB | más de 5.4 GB |

En la página de 522 se muestreó una de cada cinco páginas (105 muestras).

### Mediciones en el navegador (WebAssembly, Chromium sin pantalla)

| Medida | Casa | Taller | Grande |
|---|---|---|---|
| Componer, primera vez / otra vez | 647 / 198 ms | 737 / 239 ms | 3,285 / 2,321 ms |
| PDF de lo compuesto | 109 ms | 164 ms | 1,052 ms |
| Una página a imagen: promedio / máximo | 13.4 / 67 ms | 12.9 / 79 ms | 6.8 / 179 ms |
| Memoria de WebAssembly con el documento compuesto | 25 MiB | 36 MiB | 384 MiB |
| Todas las páginas a imagen, como hoy: tiempo y memoria de WebAssembly | 0.9 s, 97 MiB | 1.1 s, 155 MiB | 6.8 s, 2,873 MiB |
| App web: la pestaña se congela al entrar a Memoria | 1.4 s | 1.7 s | 13.3 s |
| App web: memoria que agrega Chromium | 175 MiB | 278 MiB | 5,215 MiB |

La primera composición incluye leer las fuentes. Con el proyecto grande la app
web terminó en esta máquina (7 GB), con 13 segundos congelada y 5.2 GB más.
En un teléfono no se probó; con 4 a 8 GB de memoria compartida con el sistema,
lo esperable es que el navegador cierre la pestaña, pero **no está medido**.

### Las cuatro memorias, por separado

| Capa | Casa | Taller | Grande | Comentario |
|---|---|---|---|---|
| Archivo comprimido: PDF | 184 kB | 328 kB | 7.0 MB | Ya se genera de la misma composición |
| Archivo comprimido: PNG de cada página | 216 kB por página, 2.7 MiB todas | 247 kB, 5.3 MiB | 253 kB (máx. 491), 129 MiB | Codificar tarda 18 ms por página, más que volver a convertirla (5–8 ms). No conviene guardarlas así |
| Decodificada (RGBA en memoria) | 62 MiB | 104 MiB | 2,470 MiB | Hoy todas a la vez, más la copia de egui al subirlas |
| Texturas | 62 MiB | 104 MiB | 2,470 MiB | 4.73 MiB por página a 1.6 px/pt |
| Documento compuesto (Typst) | 26 MiB | 38 MiB | 385 MiB | No lo cubre ningún tope de imágenes |

### Estrategia propuesta (no implementada en R1)

1. **Componer una vez por revisión del proyecto**, al entrar a Memoria o al
   pulsar «Actualizar», nunca en cada tecla, con el aviso «Componiendo…». En
   el navegador, componer 522 páginas tarda 3.3 s; en escritorio, 1.3 s.
2. **Convertir a imagen solo las páginas visibles y una de cada lado**, una
   a la vez, sin guardar la imagen decodificada después de subirla. Cambiar de
   página cuesta de 5 a 7 ms por página (la más lenta medida tardó 179 ms en
   el navegador).
3. **Tope de texturas como objetivo de diseño**: 6 páginas a 1.6 px/pt son
   28.4 MiB, dentro del objetivo de unos 30 MiB. Es una meta: el límite real
   existirá cuando esté implementado y medido en R5.
4. **El documento compuesto queda fuera de ese tope**: para 522 páginas son
   unos 385 MiB. La proyección para el proyecto grande es de unos 0.4 GB en
   total contra más de 5.4 GB hoy. Si se quiere bajar más, la opción es
   componer por partes; no está evaluada.
5. **No guardar PNG ni otra capa comprimida**: cuesta más codificar que
   volver a convertir la página.
6. **El PDF no cambia**: sale de la misma composición, como hoy.

## 5. Teclado virtual en el teléfono

**No hubo teléfono real disponible.** No se probó Safari en iOS ni Chrome en
Android en un dispositivo. Lo que sigue separa lo ejecutado en emulación, lo
que sale de leer el código de eframe y lo que queda por probar a mano.

### Pruebas ejecutadas (Chromium sin pantalla, emulación de Pixel 7)

412 × 839 px, toque, agente de Android. Escala 1: con la escala real del
teléfono (2.625) Chromium emulado informa el tamaño del lienzo en píxeles CSS,
eframe lo arma a un tercio de resolución y todo sale 2.6 veces más grande.
Parece un defecto de la emulación; hay que confirmarlo en un teléfono.

| Prueba | Resultado |
|---|---|
| Un toque en un campo de texto le da el foco (el `<input>` oculto de eframe) | Funciona al primer toque |
| Lo escrito aparece en el campo y en el proyecto | Funciona |
| La página no se desplaza al enfocar (el lienzo es fijo) | Funciona: desplazamiento 0 |
| Teclado simulado: el alto de la ventana baja de 839 a 503 px, como en Android con `resizes-content`, con un campo que sigue a la vista («Nombre» del tablero) | El lienzo se ajusta, el panel de abajo se reduce, el campo conserva el foco y se puede seguir escribiendo |
| Lo mismo con un campo que el teclado deja fuera («Longitud», abajo del panel) | **Falla**: el campo sale de la vista y el `<input>` oculto pierde el foco. Las teclas siguen llegando al campo (la longitud cambió de 12 a 125 m), pero no se ve. En un teléfono, perder el foco cerraría el teclado |
| Campo a medio tapar en el borde inferior, sin teclado | Tocarlo no lo trae a la vista; la primera tecla sí (egui sigue al cursor al escribir) |
| Al cerrar el teclado (la ventana vuelve a 839 px) | El panel de abajo se queda del alto reducido; se puede arrastrar su borde |
| Recargar la página justo después de escribir | El dato se conserva |
| Aviso «pensado para pantallas grandes» | Tapa la barra de Siguiente hasta pulsar «Entendido» (se queda hasta R7) |

### Análisis del código de eframe 0.36 (no comprobado en dispositivo)

- **iOS Safari, primer toque.** Safari abre el teclado solo si el foco se da
  dentro del mismo toque. eframe da el foco al `<input>` oculto en el cuadro
  siguiente, y en el toque solo lo vuelve a enfocar si ya lo tenía. Lo
  probable es que el primer toque no abra el teclado y el segundo sí.
  Limitación del framework.
- **Modo normal del teclado** (Android Chrome desde la versión 108 y Safari).
  El teclado tapa la página sin cambiar el tamaño del lienzo, y egui no se
  entera. Un campo en la mitad de abajo puede quedar tapado. El navegador
  puede desplazar la vista hacia el `<input>` oculto, que eframe coloca en el
  cursor, y eso podría mostrarlo. No se sabe sin probar. Limitación del
  navegador y del framework.
- **Por qué se pierde el foco.** egui solo informa la posición del cursor
  (lo que eframe usa para mantener el `<input>` oculto enfocado) mientras el
  campo está a la vista. Si el campo enfocado queda fuera, eframe enfoca el
  lienzo en su lugar. Limitación del framework; R6 tendrá que llevar el campo
  enfocado a la vista antes de que eso pase.
- **`interactive-widget=resizes-content`** en la etiqueta viewport haría que
  Chrome y Firefox en Android cambien el tamaño del lienzo, que es el caso
  simulado: funciona con campos que quedan a la vista y falla con los que el
  teclado tapa. Safari lo ignora. No se agregó en R1.
- **Guardar al cerrar.** eframe registra su guardado con el evento
  `onbeforeunload`, que no existe (el nombre correcto es `beforeunload`), así
  que nunca corre. En Chromium el dato se conserva porque eframe guarda al
  perder el foco, y recargar dispara ese evento. En Safari no está comprobado.

### Procedimiento para probar en un teléfono

Con un iPhone (Safari) y un Android (Chrome), en `ieestudio.com/app`:

1. Abrir la app, pulsar «Entendido» en el aviso y elegir «Proyecto de
   ejemplo». Anotar si la interfaz se ve nítida y del tamaño de una app
   normal, o borrosa y muy grande (el caso de la escala emulada).
2. Paso Proyecto: tocar «Nombre del proyecto» **una vez**. Anotar si abre el
   teclado al primer toque o hasta el segundo.
3. Escribir «Prueba». Anotar si las letras aparecen, si el autocorrector las
   duplica o las borra y si funciona borrar.
4. Paso Tableros: tocar «Longitud» en la ficha de abajo. Anotar si el campo
   queda visible arriba del teclado; si no, escribir un número y ver si
   aparece.
5. Con el teclado abierto, girar el teléfono y regresarlo. Anotar si la
   interfaz se reacomoda.
6. Cerrar el teclado (botón «Listo» o atrás) y anotar si la interfaz recupera
   su tamaño.
7. Recargar la página. Anotar si «Prueba» sigue en el nombre del proyecto.
8. Paso Revisar: anotar si todo se ve sin encimarse.

Para cada paso, anotar navegador, versión del sistema y si funcionó, falló o
funcionó con un rodeo. Una falla en los pasos 2 a 4 es del framework si
también ocurre en la demostración de egui (`egui.rs`); si no, es de
IEEStudio.

## 6. Versión web: actualización

Prueba con un servidor que responde como GitHub Pages (caché de 10 minutos):
se publica una versión 2 con una marca en `index.html` y se revisa qué ve el
usuario.

| | Antes | Después |
|---|---|---|
| Archivos que el service worker nuevo pide al servidor | solo `sw.js` y el `.wasm` | los 11 archivos |
| `index.html` guardado con la versión 2 | **el anterior** | el nuevo |
| Al recargar, la página es la versión 2 | **No** | Sí |
| Aviso de versión nueva | No existe | «Hay una versión nueva de IEEStudio · Recargar» |

Antes, la versión nueva guardaba el `.js` y el `index.html` viejos, tomados
de la caché del navegador, junto al `.wasm` nuevo. Es decir, se mezclaban
dos versiones. La corrección pide cada archivo al servidor
(`cache: "reload"`). El aviso aparece solo si ya había una versión instalada,
busca actualizaciones al volver a la pestaña y nunca recarga solo. El
proyecto guardado en el navegador no se toca.

## 7. Rama `decision-0031-punto-a-punto`

**Se conserva.** Tiene un commit (`b68c41e`, «Decisión 0031 aceptada: P-1 a
P-6 aprobados») que no está en `main`. `git cherry` lo marca como exclusivo y
mezclarlo hoy da conflictos en `0029` y `0031`, porque `main` reescribió
después esos mismos textos. Lo más probable es que su contenido ya esté
superado, pero no es idéntico. Antonio decide si se borra.

## 8. Pendientes y recomendaciones para R2

1. Probar el procedimiento de la sección 5 en un iPhone y un Android antes de
   diseñar R6 sobre egui en el teléfono. Es lo único de R1 que no se pudo
   ejecutar.
2. Agregar `tools/capturas.sh` a la integración continua en Linux. Hoy se
   corre a mano y tarda unos 2 minutos.
3. Revisar con 386 circuitos tarda de 11 a 14 ms por cuadro. Medir por partes
   y, si es la miniatura del unifilar, cachear su dibujo en R5 o antes.
4. La memoria por páginas visibles (sección 4) en R5. Medir el tope real y el
   tiempo de composición en el navegador; considerar componer en un *worker*
   si 3 s congelado resulta molesto.
5. Versión visible en la web (SHA corto en «Acerca de»), pendiente de la
   etapa 4 del plan 1.0.
6. R2 puede usar `layout.rs` como semilla de la función de cortes 1280/900/600
   de la maqueta.
