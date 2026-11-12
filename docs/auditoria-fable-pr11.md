# IEEStudio — Auditoría independiente de las correcciones del motor eléctrico (PR #11)

11 de noviembre de 2026. Auditado: `main` en `d4665fb` (PR #11, commits `7a2f0fb` y `c10b4a3` de Opus 5.5). **No se modificó el repositorio**: los casos propios se corrieron desde una carpeta aparte de la sesión.

## Dictamen: APROBADO CON OBSERVACIONES

Las correcciones de Opus son correctas en lo que tocaron y no hay que revertir nada. Pero el estudio de falla **no queda cerrado**: hay tres observaciones (H-01 a H-03) con las que el programa puede mostrar «Cumple» y «Conforme» en la capacidad interruptiva sin que la falla estimada sea de verdad la mayor posible. Deben resolverse antes del rediseño de interfaz y antes de la 1.0.

Cada hallazgo indica su tipo de evidencia: **demostrado** (lo reproduje), **riesgo potencial** (depende de cómo capture el usuario) o **mejora**.

## 1. Evaluación general

Opus modificó 27 archivos (+3 469 / −772): el cálculo en `ieestudio-project` (`fault.rs` casi reescrito, `evaluation.rs` y `review.rs` nuevos, `calc.rs`, `transformer.rs`, `model.rs`, `motors.rs`, `verdict.rs`), la interfaz (`guided.rs`, `properties.rs`, `widgets.rs`, `state.rs`), las salidas (unifilar, memoria, línea de comandos) y las decisiones 0026–0028.

- **Los 16 puntos que su informe marca «Corregido» sí lo están.** Los revisé en el código y los comprobé con casos propios.
- **Su informe es honesto en lo que ejecutó:** reproduje las 175 pruebas, el formato y los dos `clippy`.
- **Su punto débil es la clasificación.** Dejó como «Baja / pendiente» la falla de fase a neutro en 240/120 V (su hallazgo 18) mientras el programa sigue afirmando «es una cota superior». Con la definición de este encargo, eso es crítico.
- **El método no se validó contra norma.** Ni él ni yo verificamos el texto de la IEC 60909, la NMX-J-116 o el Cuaderno 158; `k = 4` y la tolerancia del 10 % siguen siendo criterios de ingeniería del perfil.

## 2. Problemas resueltos satisfactoriamente

| Corrección de Opus | Cómo lo comprobé |
|---|---|
| Motores por zona de tensión y paso a través del transformador | Caso de tres zonas en cascada (480 → 220 → 480 V) cotejado en ohms: 118.861 A, 230.861 A y 103.624 A, idéntico al programa |
| Un transformador sin datos nunca hereda la falla de su primario | 20 000 proyectos aleatorios, 0 casos |
| Estudio por partes: la falla de un intermedio no depende de la fuente | Mismo barrido, 0 diferencias al cambiar la fuente |
| Corriente `S / V` en transformadores de una o dos fases, igual que su dimensionamiento | 25 kVA 480 → 240 V: 52.08 A y 104.17 A |
| Sin valores inventados en transformadores, acometida y datos de catálogo | Lectura de código; los archivos anteriores con números se leen igual |
| Errores con su causa en vez de descartarse | Casos con potencia, impedancia o conexión faltante o inválida |
| Cuatro estados y «Cómo se obtuvo» iguales en Revisar, memoria y línea de comandos | Capturas de Revisar y texto extraído de 3 memorias en PDF (salvo H-11) |
| Nunca «No cumple» ni «Conforme» con capacidad interruptiva pendiente | Barrido aleatorio: 1 306 grupos «Cumple», todos válidos y con todos sus interruptores arriba de la falla |
| Sistema de las barras en un solo lugar | El unifilar ya delega en `connection::bus_system` |

## 3. Problemas que siguen presentes

### H-01 · Crítico · demostrado — La falla de fase a neutro en 240/120 V puede superar el valor que el programa llama «cota superior»

- **Dónde:** `fault.rs` líneas 384–393 (`transformer_voltage`) y 776–829 (`Topology::base`); `transformer.rs` 27–52; texto del dictamen en `verdict.rs` 198–200.
- **Hoy:** en un secundario monofásico de tres hilos solo se calcula la falla entre líneas, `S / (240 · Z)`, y se presenta como la corriente mayor posible.
- **Escenario:** acometida `1F-3H 240/120 V`, transformador de 25 kVA y 3 %, interruptores de 5 kA. El programa da **3.86 kA**, «Cumple» y «Conforme con NOM-001-SEDE-2012… la falla estimada es una cota superior». Lo mismo con un intermedio seco 480 → 240/120 V.
- **Consecuencia:** en las terminales de un transformador con derivación central, la falla de línea a neutro es de 1.33 a 1.67 veces la de línea a línea (1.5 en el método punto a punto de Bussmann). Aquí serían **5.1 a 6.4 kA**: un interruptor de 5 kA aparece como conforme sin serlo.
- **Corrección:** decisión D-1. Como mínimo, dejar de declarar «Cumple» en estos secundarios.
- **Prueba:** el caso anterior debe dar no menos de 1.5 × 3.858 kA o quedar «No concluyente»; nunca «Cumple» con 5 kA.

### H-02 · Alto · riesgo potencial — El dato de la suministradora no dice a qué tensión corresponde

- **Dónde:** `fault.rs` 483–498 (`source_base`); `properties.rs` 952–955.
- **Hoy:** «Falla disponible» manda sobre el transformador de la acometida y se aplica tal cual en baja tensión. No se compara con el bus infinito.
- **Escenario:** subestación propia de 1 000 kVA, 5.75 %, 480 V (bus infinito 23.24 kA). El usuario captura los 5 kA que CFE da en 13.2 kV. El programa usa 5 kA en 480 V y da «Cumple» con interruptores de 10 kA. Con ese dato de media tensión el secundario tendría unos **19.9 kA**. Al revés, un dato de 50 kA (imposible en ese secundario) se acepta sin aviso.
- **Consecuencia:** resultado del lado inseguro si el dato es de media tensión; contradicción no detectada en el otro sentido.
- **Corrección:** decisión D-2.
- **Prueba:** dato de 5 kA en media tensión + transformador → cerca de 19.9 kA en el secundario; dato mayor que el bus infinito → aviso.

### H-03 · Alto · demostrado — Una carga que el propio programa sospecha que es motor no impide el «Cumple»

- **Dónde:** `review.rs` 96–114; `fault.rs` 723–737 (solo cuentan los tipos Motor y Aire acondicionado); `guided.rs` 2455 y 2554.
- **Escenario:** dos «Bomba 50 hp» como «Otra carga», dato de 9.8 kA, interruptores de 10 kA: «Cumple», «Conforme», «Por resolver · 0». Capturadas como motor: **10.84 kA, no concluyente**.
- **Consecuencia:** el programa afirma «cota superior» con una aportación que él mismo señala como probable. Además «Es otra carga» solo se recuerda mientras la app está abierta, y la memoria y la línea de comandos repiten el aviso siempre.
- **Corrección:** decisión D-3.
- **Prueba:** con un aviso de este tipo sin confirmar, la capacidad interruptiva de esa zona es «No concluyente»; confirmado y guardado, vuelve a evaluarse.

### H-04 · Medio · demostrado — Tres criterios distintos para decidir si un sistema es trifásico

- **Dónde:** `fault.rs` 384–393 (relación a menos de 0.1 de 2), `system.rs` 52–62 (punto medio entre √3 y 2), `calc.rs` 316–321.
- **Escenario:** sistema 226/120 V (relación 1.88), 25 kVA, 3 %. El modelo lo trata como monofásico de tres hilos y la falla lo calcula como trifásico: **2.37 kA en vez de 4.10 kA**, y «Cumple». Por lectura del código, pasa igual con relaciones de 2.1 o más.
- **Alcance:** la interfaz solo ofrece seis sistemas fijos; se llega por archivo `.iees` o por el servidor MCP.
- **Corrección:** que `transformer_voltage` use `System::has_three_phases`. **Prueba:** ese sistema da 4.10 kA.

### H-05 · Medio · demostrado — Datos contradictorios que no se detectan

- **Dónde:** `fault.rs` 562–563 y 510–527.
- **Escenarios:**
  - Conexión «Monofásico» en el transformador de la acometida de un sistema 220Y/127 V con impedancia de tabla: busca el renglón monofásico (1.5 %) pero calcula la corriente como trifásico. La falla pasa de 9.84 a 13.12 kA sin aviso.
  - Impedancia de 30 % (por 3.0): 0.73 kA y «Conforme». Ningún aviso de valor fuera de lo normal.
- **Corrección:** avisar cuando la conexión contradice al sistema y cuando la impedancia sale del intervalo de la tabla de su clase. **Prueba:** un caso por aviso.

### H-06 · Medio · demostrado — Capacidad del transformador de la acometida sin comparar con la carga

- **Dónde:** `calc.rs` 1145–1151: la verificación «Capacidad del transformador» solo existe para el intermedio.
- **Escenario:** 300 kW en un transformador de acometida de 75 kVA: «Conforme».
- **Corrección:** decisión D-5. **Prueba:** ese caso deja un aviso o un «No cumple», según se decida.

### H-07 · Medio · demostrado — El dictamen dice «Conforme con NOM-001-SEDE-2012» sin el «en lo evaluado»

- **Dónde:** `verdict.rs` 127–132 y 198–203. Viene de la decisión 0023, no de Opus.
- **Hoy:** en cuanto hay falla estimada se borra la lista de fuera de alcance y el texto pasa a «Conforme con la norma: todo cumple y nada quedó sin evaluar», aunque el propio módulo dice que la conformidad es siempre en lo evaluado.
- **Corrección:** decisión D-6. **Prueba:** ningún texto del dictamen afirma conformidad con la norma sin ese límite.

### H-08 · Medio · sigue pendiente — Datos iniciales supuestos en circuitos y alimentadores

- **Dónde:** `state.rs` 88, 335 y 351 (1 000 W y 10 m). Es el hallazgo 19 de Opus; contradice el criterio de la 0026 que él mismo aplicó a los transformadores.
- **Corrección:** que nazcan vacíos y sin calcular. **Prueba:** un circuito nuevo queda «Sin evaluar» hasta capturar potencia y longitud.

También siguen abiertos, como él reportó: la conexión propia del secundario (su 17) y los cuatro estados en el resto de las verificaciones (su 20).

### H-09 · Medio · demostrado — El programa anterior ya no abre los archivos nuevos

- **Dónde:** `model.rs` 21 y 344–359. El formato sigue en 3 y su comentario dice que «un programa anterior abre sin error lo que escribe este».
- **Escenario:** proyecto con un transformador sin potencia, guardado con la versión actual y leído con el código anterior al PR: `missing field kva`. Con todos los datos capturados sí abre.
- **Corrección:** decisión D-7. **Prueba:** lectura cruzada entre versiones.

### H-10 · Medio · demostrado — El unifilar escribe notas que no corresponden

- **Dónde:** `ieestudio-diagrams/src/lib.rs` 1394–1406 y 705–720.
- **Escenarios:**
  - Sin ninguna falla estimada (dato inválido o transformador sin potencia), la nota dice «corriente de falla con el dato de la suministradora… es una cota superior».
  - Con impedancia de tabla, dice «impedancia de placa reducida 10 %», que no se aplicó.
  - Un tablero sin falla estimada no lleva ninguna indicación en el plano.
- **Corrección:** armar la nota con el origen real de cada zona y escribir «Icc SIN ESTIMAR» con su causa. **Prueba:** tres casos sobre el texto del plano.

### H-11 · Medio · demostrado en código — «Dato inválido» se pierde en cuatro lugares

- **Dónde:** `guided.rs` 636, `properties.rs` 364 y 1049, `ieestudio-reports/src/lib.rs` 648. Solo distinguen «falta un dato».
- **Hoy:** un dato inválido de la acometida aparece como «sin datos de falla… fuera de alcance» y se omite en «Barras de…» de la memoria. «Cómo se obtuvo» sí lo muestra bien.
- **Alcance:** los campos de la interfaz no aceptan cero ni negativos; llega por archivo.
- **Corrección:** tratar los tres tipos en esos cuatro sitios. **Prueba:** dato de 0 kA → «Dato inválido» en todos.

### Menores (Bajo)

| # | Dónde | Qué pasa | Corrección y prueba |
|---|---|---|---|
| H-12 | `fault.rs` 758–761 y 1384–1396 | Los motores detrás de un transformador sin datos cuentan como «1 motor sin datos» en el lugar, pero el total del estudio dice 0 y la sección «Motores sin datos» de la memoria no los lista | Un solo conteo; prueba con ese caso |
| H-13 | `references.csv` 51 | La regla de motores todavía dice «en cada tablero» | Actualizar el texto |
| H-14 | `fixtures/` | Las reglas nuevas (paso por el transformador, una o dos fases) no tienen caso de referencia; `reviewed_by` sigue vacío | Agregar los casos del cotejo |
| H-15 | `motors.rs` 341 | La nota del compresor dice que su corriente estimada «solo se usa para la caída y el cuadro», pero también entra en la falla | Corregir la nota |
| H-16 | `review.rs` 146–148 | Un transformador de aislamiento 127 → 127 V desde 220Y/127 V no se marca como 1:1 | Comparar las tensiones de la conexión |
| H-17 | `widgets.rs` 220–236 | Un valor rechazado (0, negativo, texto) se ignora sin decirlo | Marcar el campo |
| H-18 | `fault.rs` 1074–1079 | Las impedancias se suman como magnitudes; no es estrictamente cota superior (diferencia menor a 0.2 %) | Documentarlo |

## 4. Pruebas ejecutadas y resultados

| Prueba | Resultado |
|---|---|
| `cargo test --workspace` | 175 pasan, 0 fallan |
| `cargo fmt --all -- --check` | Sin cambios |
| `cargo clippy --all-targets -- -D warnings` | Sin avisos |
| `cargo clippy --target wasm32-unknown-unknown -p ieestudio-app -- -D warnings` | Sin avisos |
| 21 proyectos propios, con cotejo a mano donde hay número, y 3 lecturas de archivos anteriores | Reproducen H-01 a H-06, H-10, H-12 y H-16; confirman la sección 2 |
| Barrido aleatorio: 20 000 proyectos con datos faltantes e inválidos | 0 caídas, 0 violaciones |
| Línea de comandos: `calcular`, `memoria` y `dxf` en 5 casos | Sin errores; 6 memorias en PDF generadas, texto leído en 3 |
| Tres capturas del paso Revisar | Coinciden con el cálculo |
| Lectura con el código anterior al PR | Reproduce H-09 |

**No ejecuté:** la integración continua en Windows, el instalador, el sitio web, el uso interactivo de la interfaz ni la vista avanzada. **No verifiqué** ningún valor contra el texto de la NOM, la NMX-J-116 ni la IEC 60909.

## 5. Pruebas adicionales recomendadas

1. Las de cada hallazgo, como pruebas del repositorio.
2. El barrido aleatorio como prueba permanente.
3. Un caso de referencia en `fixtures/` por cada regla de la 0028, firmado.
4. Lectura cruzada de archivos entre la versión publicada y la actual.
5. Un proyecto real con memoria conocida, de punta a punta.

## 6. Decisiones de ingeniería que requieren aprobación

| # | Decisión | Alternativas | Recomendación |
|---|---|---|---|
| D-1 | Falla de fase a neutro en secundarios con derivación central | (a) multiplicar por un factor del perfil (1.5); (b) dejarla «No concluyente»; (c) solo quitar el texto «cota superior» | (a), con (b) mientras se aprueba |
| D-2 | Tensión del dato de la suministradora | (a) indicar si es del secundario o de media tensión y pasarlo por el transformador; (b) aceptarlo solo en baja tensión y avisar si contradice | (a) |
| D-3 | Cargas que parecen motor | (a) «No concluyente» hasta confirmar, guardando la confirmación en el proyecto; (b) solo aviso | (a) |
| D-4 | Falla en el secundario de un intermedio (hoy 72.9 kA donde con la fuente conocida serían 9.49 kA) | (a) seguir con bus infinito; (b) usar la falla de arriba cuando se conoce, que sigue siendo cota superior | (b); cambia la 0028 |
| D-5 | Capacidad del transformador de la acometida | (a) verificarla si es propio; (b) solo aviso; (c) no revisar | (a); requiere el dato «propio o de CFE» |
| D-6 | Texto del dictamen | Siempre «Conforme en lo evaluado» | Sí |
| D-7 | Formato del archivo | (a) subir a formato 4; (b) escribir los vacíos de modo que el anterior los lea | (a) |
| D-8 | Tabla de la NMX-J-116 y si su mínimo ya incluye la tolerancia | Cotejar con el texto de la norma | Pendiente de Antonio |

## 7. Dictamen final

**APROBADO CON OBSERVACIONES.** Las correcciones de Opus se quedan. Antes de continuar: resolver H-01, H-02 y H-03 (con D-1 a D-3) y, en el mismo paso, H-04, H-09, H-10 y H-11, que son cambios chicos. Mientras tanto, un «Cumple» de capacidad interruptiva no debe tomarse como concluyente en sistemas 240/120 V, en acometidas con transformador y dato de la suministradora, ni en proyectos con avisos de «parece motor».

**Fuente del factor de fase a neutro:** método punto a punto de Bussmann, nota sobre transformadores monofásicos con derivación central (1.5; teórico de 1.33 a 1.67, con 1.5 × %R y 1.2 × %X).
