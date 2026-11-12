# Auditoría del modelo eléctrico y del estudio de falla

Noviembre de 2026. Revisión del modelo de conexiones, los transformadores, el
estudio de falla por bus infinito (decisión 0023), la aportación de motores y
la forma en que se informan los resultados incompletos. Las correcciones y el
método nuevo de motores quedan en la decisión 0026.

## Principio

Un dato eléctrico se captura **una sola vez**, en el elemento y el nivel de la
instalación al que pertenece, y los elementos que dependen de él lo obtienen
del modelo de conexiones. Ningún dato desconocido se reemplaza por un valor
que parezca válido.

## Cómo está armado el modelo

- `Source.system` es la tensión de servicio. `Source.substation` es el
  transformador de la acometida (CFE o propio); su secundario es
  `Source.system`, así que no se vuelve a capturar.
- `Panel.transformer` es un transformador **intermedio**: su primario es el
  alimentador del tablero, a la tensión del tablero de donde sale, y su
  secundario fija la tensión del tablero y de todo lo que cuelga de él.
- `connection::bus_system` deduce el sistema de las barras de cada tablero:
  el del transformador intermedio más cercano aguas arriba o el de la fuente.
  Los circuitos no preguntan su sistema.
- El estudio de falla recorre los tableros de arriba hacia abajo.

## Hallazgos

Gravedad: **crítica**, el resultado puede quedar del lado inseguro o mostrar un
número que no corresponde; **alta**, se ocultan datos faltantes o se inventan;
**media**, inconsistencias que pueden volverse errores; **baja**, limitaciones
que solo se documentan.

| # | Gravedad | Hallazgo | Dónde |
|---|---|---|---|
| 1 | Crítica | La falla de un transformador intermedio se calculaba como trifásica siempre que su secundario tuviera tensión entre fases, aunque alimentara una conexión de una o dos fases: `In = S / (√3 · V)` en lugar de `S / V`. La falla salía √3 veces menor (−42 %), del lado inseguro. El dimensionamiento del mismo transformador sí usaba la conexión, así que los dos cálculos se contradecían. | `fault.rs`, `transformer_voltage` |
| 2 | Crítica | La aportación de **todos** los motores del proyecto se sumaba en **todas** las barras, aunque estuvieran a otra tensión o detrás de un transformador: se sumaban amperes de 220 V con amperes de 480 V. | `fault.rs`, `with_motors` |
| 3 | Crítica | Si los datos de un transformador intermedio no servían, sus barras heredaban la corriente de falla del primario, que está a otra tensión, y la presentaban como propia. | `fault.rs`, `Err(_) => upstream` |
| 4 | Alta | Al activar un transformador se creaban 45 kVA, 3 %, secundario 220Y/127 V y 3 m de conductores; al activar la subestación, 112.5 kVA, 3 % y 13.2 kV. Eran resultados con apariencia válida sin dato del usuario. En una acometida 220Y/127 V el valor por omisión dejaba un transformador de 220 V a 220 V: la huella de capturar dos veces el mismo equipo. | `properties.rs` |
| 5 | Alta | Sin dato de la suministradora ni de la acometida el estudio entero se cancelaba (`source_fault(...)?`), incluso la falla de un transformador intermedio que sí tenía datos y que por bus infinito no depende de la fuente. | `fault.rs`, `study` |
| 6 | Alta | Los errores de la acometida se descartaban (`substation_impedance(...).ok()?`, `steps_for_transformer(...).ok()?`): la pantalla solo decía «sin datos de falla». | `fault.rs`, `source_fault` |
| 7 | Alta | Un motor quedaba fuera de la aportación si su circuito no se calculaba, aunque su corriente sí se conociera, o si le faltaba la potencia; nada avisaba que la falla estaba incompleta. Un motor nuevo nacía con 1 hp, así que nunca aparecía como incompleto. | `fault.rs`, `motor_shares`; `state.rs` |
| 8 | Alta | «Agregar tablero» lo creaba siempre trifásico, aunque el sistema fuera 240/120 V o el tablero padre fuera de una fase: el tablero nacía con una conexión imposible. | `state.rs`, `new_panel` |
| 9 | Media | El texto decía «bus infinito» y «cota superior» también cuando la falla era el dato de la suministradora: dos hipótesis distintas presentadas igual. | `fault.rs`, textos |
| 10 | Media | El sistema de las barras se deducía en tres lugares con tres copias del mismo código (cálculo, conexiones y unifilar). | `calc.rs`, `connection.rs`, `diagrams` |
| 11 | Media | Ningún aviso cuando el secundario de un transformador intermedio tiene la misma tensión que su primario, señal de que se capturó dos veces el transformador de la acometida. | — |
| 12 | Media | La sospecha de que una «Otra carga» es un motor (por su nombre: bomba, extractor) la calculaba la interfaz, solo para la vista guiada (pasos Circuitos y Revisar): no llegaba al resultado del proyecto, la memoria ni la línea de comandos. | `guided.rs`, `suspicious_kinds` |
| 13 | Media | Si fallaba la regla del 125 % del transformador o la del alimentador con motores (430-24), el tramo se dimensionaba sin ella y sin avisar (`.ok()`). El 125 % estaba escrito en el código como respaldo de la tabla. | `calc.rs`, `transformer.rs` |
| 14 | Baja | El secundario de un transformador intermedio usa la conexión de su primario: uno alimentado de fase a neutro (277 V) no puede dar circuitos de 240 V. Falta un dato de conexión del secundario en el modelo. | `connection.rs` |
| 15 | Baja | En un secundario 120/240 V la falla de fase a neutro puede superar a la de fase a fase; la 0023 solo estima la trifásica o la de fase a fase. | 0023 |
| 16 | Baja | Una longitud de 0 m se acepta como válida, y los circuitos y alimentadores nuevos nacen con 1000 W y 10 m. Es el mismo principio del punto 4 en toda la captura; se deja para el rediseño de la captura. | `state.rs`, `exchange` |

## Lo que dicen las normas sobre el método

La NOM-001-SEDE-2012 exige que la capacidad interruptiva alcance la corriente
de falla disponible (110-9, 110-10), pero no fija cómo calcularla. Los métodos
reconocidos son:

- **IEC 60909-0** (edición 2016, sustituida en julio de 2026 por la edición 3).
  Modela la red con una impedancia equivalente obtenida de su corriente de
  falla, `ZQ = c · UnQ / (√3 · I"kQ)`, referida al otro lado de cada
  transformador con el cuadrado de la relación; los motores asíncronos son
  fuentes detrás de una impedancia `ZM = UrM² / (SrM · ILR/IrM)`; para grupos
  de motores de baja tensión admite `ILR/IrM = 5`. La aportación de un motor
  llega a otra tensión a través de la impedancia del transformador.
- **Schneider Electric, Cuaderno técnico 158** (cálculo de corrientes de
  cortocircuito con el método de impedancias de la IEC 60909): un motor
  asíncrono es una fuente con impedancia subtransitoria de 20 a 25 % (4 a 5
  veces su corriente); para llevar su aportación al otro lado de un
  transformador, como estimación conservadora, basta multiplicarla por la
  relación de tensiones; despreciar la impedancia de la red es aceptable
  cuando su potencia de cortocircuito es muy grande frente a la del
  transformador.
- **Eaton Bussmann (Neeser, 2018), método punto a punto**: 4 veces la
  corriente a plena carga de los motores **a la tensión de las barras**; los
  motores de 208 V y 120 V no se suman en la acometida de 480 V.

Las tres coinciden en que no se suman directamente motores de tensiones
distintas.

## Correcciones

En la decisión 0026 están el método y las razones. En resumen:

- Datos de transformadores y acometida sin valores inventados: los que faltan
  quedan vacíos y el resultado dice cuál falta.
- La falla de cada transformador usa la misma corriente nominal que su
  dimensionamiento.
- El estudio se hace por partes: lo que tiene datos se evalúa; lo que depende
  de una fuente sin datos queda fuera de alcance; lo que tiene datos
  incompletos queda sin evaluar con su causa.
- La aportación de motores sigue la topología: los de la misma zona de
  tensión, completos; los de otra zona, a través de la impedancia del
  transformador.
- Los motores sin datos se listan y hacen no concluyente el resultado que
  afectan.
- Las cargas que parecen motor se informan en el resultado del proyecto.
- Un solo cálculo del sistema de las barras.
