# Etapa 0 del plan 1.0: línea base y casos reproducibles

11 de noviembre de 2026. Base revisada: `main` en `35cf49e` (PR #12, que
agregó el plan nuevo y la auditoría de Fable; desciende de `d4665fb`, PR #11).
**No se modificó el motor de cálculo ni la interfaz**: esta etapa agrega
pruebas, casos, un script y documentos.

## 1. Cómo repetirlo

```
tools/auditoria.sh             # hallazgos abiertos y estado de cada caso
tools/auditoria.sh anterior    # además, compatibilidad con la versión ef59295
```

- `crates/ieestudio-project/tests/auditoria_pr11.rs`: casos del motor.
- `crates/ieestudio-cli/tests/auditoria_salidas.rs`: unifilar, memoria y
  línea de comandos.
- `fixtures/auditoria-pr11/`: un `.iees` por caso, para abrirlo en la app.
- `fixtures/compatibilidad/`: archivos de versiones anteriores.

Cada prueba con `#[ignore]` describe el comportamiento corregido y hoy
falla con lo que muestra el programa. Al corregir el hallazgo se le quita el
`#[ignore]` en el mismo PR. `cargo test --workspace` sigue en verde: 181
pasan, 0 fallan, 22 ignoradas (21 hallazgos abiertos y el informe).

## 2. Estado de cada hallazgo

Estados: **pendiente** (reproducido, sin corregir), **decisión** (necesita
aprobación de Antonio antes de corregir), **corregido**, **no reproducido**.

### Auditoría de Fable

| ID | Gravedad | Estado | Prueba o evidencia | Lo que muestra hoy |
|---|---|---|---|---|
| H-01 | Crítico | **Corregido** (etapa 1A, decisión 0029) | `h01_*` (7) y `h01_*` de salidas (2) | Antes: «Cumple» con 3.858 kA y «Conforme». Ahora: 6.44 kA de fase a neutro, «No concluyente» |
| H-02 | Alto | **Corregido** (etapa 1B, decisión 0029) | `h02_*` (7) y `h02_*` de salidas (2) | Antes: 5 kA sin tensión «Cumple» con 10 kA y 50 kA sin aviso. Ahora: sin ubicación «Sin evaluar»; en 13.2 kV, 22.92 kA; 50 kA en baja tensión, «Dato inválido» |
| H-03 | Alto | **Corregido** (etapa 1C, decisión 0029) | `h03_*` (5) y `h03_*` de salidas (2) | Antes: aviso «parece motor» y aun así «Cumple». Ahora: «No concluyente» hasta confirmar; «Es otra carga» se guarda en el archivo |
| H-04 | Medio | Pendiente | `h04_un_solo_criterio_de_trifasico` | 226/120 V: 2.365 kA en vez de 4.097 kA |
| H-05 | Medio | Pendiente | `h05_*` (2) | «Monofásico» en 220Y/127 V: 13.12 kA sin aviso; Z = 30 %: 1.09 kA sin aviso |
| H-06 | Medio | Decisión D-5 | `h06_*` | 300 kW en 75 kVA: «Conforme» |
| H-07 | Medio | Decisión D-6 | `h07_el_dictamen_dice_en_lo_evaluado` | «Conforme con NOM-001-SEDE-2012: todo cumple y nada quedó sin evaluar» |
| H-08 | Medio | Pendiente | Lectura de `state.rs` (interfaz) | Circuito nuevo con 1000 W y 10 m; alimentador nuevo con 10 m |
| H-09 | Medio | **Corregido hacia adelante** (etapa 1B, formato 4) | `tools/auditoria.sh anterior` y pruebas de `model.rs` | Ningún archivo nuevo abre a medias en una versión anterior: dice «formato 4, más nuevo», salvo 4 que fallan antes con «missing field» porque esa versión no revisaba el formato primero; desde esta versión se revisa primero |
| H-10 | Medio | Pendiente | `h10_*` (3) | Nota «con el dato de la suministradora» sin falla; «reducida 10 %» con impedancia de tabla; sin «Icc SIN ESTIMAR» |
| H-11 | Medio | Pendiente | `h11_*` (2) | Memoria y línea de comandos: «Dato inválido … sin evaluar en …» |
| H-12 | Bajo | Pendiente | `h12_*` | El lugar cuenta 1 motor sin datos; el estudio, 0 |
| H-13 | Bajo | Pendiente | `h13_*` | `references.csv` dice «en cada tablero» |
| H-14 | Bajo | Pendiente | `fixtures/engineering-cases/` | Sin casos de referencia de la 0028; `reviewed_by` vacío |
| H-15 | Bajo | Pendiente | `h15_*` | Nota «solo se usa para la caída de tensión y el cuadro de cargas» |
| H-16 | Bajo | Pendiente | `h16_*` | 127 → 127 V sin aviso de relación 1:1 |
| H-17 | Bajo | Pendiente | Lectura de `widgets.rs` (interfaz) | Un valor rechazado se ignora sin marcar el campo |
| H-18 | Bajo | Pendiente (documentar) | Cálculo en D-4 | Suma de magnitudes: hasta 0.6 % (X/R 10 y 3) o 2 % (15 y 2) por debajo |

### Pendientes de la auditoría de Opus

| # | Estado | Prueba | Lo que muestra hoy |
|---|---|---|---|
| 17 | Pendiente | `p17_el_secundario_tiene_su_propia_conexion` | Elevador 127 → 240/120 V: rechaza un circuito de 240 V |
| 18 | Corregido con H-01 | — | Etapa 1A |
| 19 | Decisión (plan 2C) | `p19_*` y H-08 | Circuito de 0 m: «Cumple» |
| 20 | Pendiente | Ver N-1 | Las verificaciones de tramos no distinguen faltante, inválido y no concluyente |

Las 16 correcciones de Opus que Fable confirmó no se volvieron a auditar una
por una; las pruebas existentes (175) siguen pasando y los casos nuevos no
contradicen ninguna.

### Hallazgos nuevos de esta etapa

| ID | Gravedad | Qué | Dónde |
|---|---|---|---|
| N-1 | Medio | Un transformador intermedio sin potencia deja en sus tramos «dato inválido: falta la potencia del transformador». Es un dato que **falta**, no uno inválido (mismo tipo que el hallazgo 16 de Opus, en otro lugar) | `h09-transformador-sin-potencia.iees`, verificaciones del alimentador de TF-1. Entra en el pendiente 20 |
| N-2 | Medio | H-09 alcanza más que lo reportado: una acometida con «impedancia de tabla» tampoco abre en la versión anterior | `fixtures/compatibilidad/README.md` |
| N-3 | Alto (entrega) | Las imágenes del unifilar en ieestudio.com (inicio, funciones y guía, paso 7) son de la versión del 14 de octubre, anterior a la decisión 0022 | `website/imagenes/app-unifilar.webp`, `website/imagenes/guia/paso-7.webp` |
| N-4 | Alto (entrega) | El service worker sirve siempre lo guardado primero; la versión nueva solo aparece en la siguiente apertura completa, y no hay forma de ver qué versión corre | `crates/ieestudio-app/web/sw.js` |
| N-5 | Medio (teléfono) | El `.wasm` publicado pesa 50.7 MB (16.6 MB comprimido) | Sitio publicado |

## 3. Contradicciones del plan con el repositorio

1. **PR #12 ya está integrado.** El plan y la auditoría de Fable están en
   `main` desde `35cf49e`; no hay un PR documental pendiente.
2. **La auditoría de Fable ya está en el repositorio** (plan, etapa 0,
   punto 3), pero no estaba enlazada desde `roadmap.md` ni desde el índice
   de decisiones. Se enlaza en este PR.
3. **`roadmap.md` todavía describe el plan anterior:** «Rediseño, fases 2 y
   3: tablet y teléfono, llevando la vista guiada a una vista por paso».
   El plan nuevo pide una sola interfaz adaptativa sin vistas Guiada y
   Avanzada separadas, y las decisiones 0019 y 0020 siguen describiendo dos
   vistas. Se corrige el roadmap; las decisiones 0019/0020 se sustituyen
   cuando se apruebe el rediseño (etapa 7), no antes.
4. **El documento del plan en el proyecto de claude.ai
   (`claude/plan-hacia-1.0.md`) es el plan anterior del 10 de noviembre**,
   con otra numeración de etapas (allí la «Etapa 3» es el teléfono). Un
   modelo que lo lea ahí trabajaría con instrucciones viejas.
5. **Comprobaciones antes de cada commit:** `CLAUDE.md` dice `cargo fmt
   --all` y `cargo test`; el plan dice `cargo fmt --all -- --check` y
   `cargo test --workspace`. No es contradicción de fondo; se usan las del
   plan.
6. **El aviso «pensado para pantallas grandes»** sigue en
   `web/index.html`, como corresponde: el plan nuevo lo deja para cuando el
   teléfono sea utilizable.
7. **Lo que el plan da por hecho y sí se cumple:** PR #11 integrado con las
   decisiones 0026–0028; CI `43148932851` en verde; última versión publicada
   `v0.3.0-alpha.3`; el unifilar se guarda por revisión y modo; la web usa el
   mismo `ieestudio-diagrams` que el escritorio.

## 4. Unifilar en el navegador

Lo que se comprobó, sin tocar el equipo de nadie ni datos locales:

| Pregunta | Resultado | Cómo |
|---|---|---|
| ¿Qué versión publica ieestudio.com? | La de `35cf49e` (PR #12): `sw.js` con versión `9ca91b796d4f` | Descarga del sitio en vivo |
| ¿Es la que compiló la integración continua? | Sí: el `.wasm` en vivo tiene el mismo SHA-256 (`9ca91b796d4f…`) que el artefacto `ieestudio-sitio` de la ejecución `43157259766` | `gh run download` y `sha256sum` |
| ¿Ese `.wasm` trae el unifilar nuevo? | Sí: contiene textos que solo existen desde la 0022 y el PR #11 («SIMBOLOG…», «BUS INFINITO», «INCOMPLETA: FALTAN MOTORES», «Abrir unifilar») | `grep` en el binario |
| ¿Hay otra dirección con otra copia? | No: `jaam08.github.io/IEEStudio/app/`, `www.` y `http://` redirigen a `https://ieestudio.com/app/` | Cabeceras HTTP |
| ¿La web dibuja con otro código? | No: en `ieestudio-diagrams` y en los paneles del unifilar no hay nada exclusivo del navegador; solo `main.rs` y `files.rs` distinguen la plataforma | Búsqueda de `cfg(target_arch = "wasm32")` |
| ¿El dibujo guardado puede ser de otro proyecto? | No: la revisión es única entre documentos (contador global) y la llave incluye el modo | `state.rs`, `panels/mod.rs` |
| ¿El modo explica la diferencia? | Parcialmente: la app abre en «Revisión» (colores de la app) y la memoria/PDF/DXF usan «Plano» (blanco y negro). Las dos son el diseño nuevo | `Shared::new` |

**Causas probables, por evidencia:**

1. **Las imágenes del sitio son del unifilar viejo (N-3).** La página de
   inicio, Funciones y la Guía (paso 7) muestran una captura del 14 de
   octubre **tomada en el navegador** («en este navegador» en la barra), con
   flechas en todas las cargas y abreviaturas «4-10, 1-10d». Quien mira el
   sitio, y no `/app`, ve ese diseño.
2. **El service worker sirve primero la copia guardada (N-4).** Al abrir
   `/app` después de una publicación, el navegador muestra la versión
   anterior; la nueva se descarga detrás y solo aparece en la siguiente
   apertura completa (en una app instalada en el teléfono, puede tardar
   días si no se cierra del todo). Además, al instalar la versión nueva usa
   la caché HTTP del navegador (`max-age=600`): si hay dos publicaciones en
   menos de 10 minutos, puede guardar archivos viejos con el nombre de la
   versión nueva. No hay ningún número de versión visible para saberlo.
3. **Revisión contra Plano.** Comparar el panel de la app (Revisión) con el
   PDF del escritorio (Plano) da dos aspectos distintos, pero los dos son el
   diseño nuevo: llevan símbolos por carga, recuadro punteado del tablero y
   «ACOMETIDA EN BAJA TENSIÓN · CFE».

**Cómo distinguirlo sin borrar nada.** En `ieestudio.com/app`, paso Revisar:
si existe el botón «Abrir unifilar», corre una versión del 11 de noviembre
o posterior. Si el unifilar tiene una flecha en cada carga y textos como
«4-10, 1-10d», es una versión vieja guardada: cerrar todas las pestañas (o
la app instalada) y abrir dos veces. **No usar «Borrar datos del sitio»**: el
proyecto del navegador vive en el almacenamiento local y se perdería; el
service worker solo borra sus cachés `ieestudio-*`, nunca el proyecto.

**Corrección propuesta (etapa 4 del plan):** identificador de versión
visible (versión y SHA corto en «Acerca de»); service worker que pida
`index.html` a la red primero y los demás archivos con la versión en la URL;
instalación con `cache: "reload"`; aviso «Hay una versión nueva · Recargar»
cuando el service worker nuevo se active; y reemplazar las imágenes del
sitio. Ninguna toca el almacenamiento local del proyecto.

## 5. Mediciones iniciales

Proyecto de ejemplo (1 tablero, 6 circuitos), línea de comandos en
*release*, Linux, 2 núcleos Xeon a 2.1 GHz; cada tiempo incluye abrir el
programa y leer el archivo:

| Medición | Resultado |
|---|---|
| `calcular` | 5–7 ms |
| `memoria` (PDF de 189 kB) | 87–104 ms |
| `dxf` / `excel` | 5 ms / 6 ms |
| `cargo test --workspace` (ya compilado) | 7 s, 181 pruebas |
| `.wasm` publicado | 50.7 MB; 16.6 MB comprimido como lo sirve el sitio |
| Ejecutable de la línea de comandos | 40.3 MB |

Estos números son con un proyecto chico; el proyecto grande de prueba
(12 tableros, 400 circuitos) y las metas por cuadro de pantalla son de la
etapa 5 del plan. No se midió el tiempo por cuadro de la interfaz.

## 6. Lo que no se hizo en esta etapa

- No se modificó el motor, la interfaz, el service worker ni el sitio.
- No se revisó la interfaz a mano ni la vista avanzada; las capturas son de
  la vista guiada a 1440 × 900.
- No se cotejó ningún valor con el texto de las normas (ver D-8).
- No se repitió la auditoría de las 16 correcciones de Opus.
