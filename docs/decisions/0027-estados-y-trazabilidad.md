# 0027. Estados de evaluación, resultados parciales y trazabilidad

**Estado:** aceptada (noviembre de 2026). Los cuatro estados los fijó el dueño
del proyecto. Por ahora se aplican al estudio de falla y a la capacidad
interruptiva; los demás tramos siguen con su estado de la 0025.

## Problema

- **«Sin evaluar» mezclaba casos distintos.** Se usaba igual para un dato
  que falta, para un dato inconsistente y para una cota que no alcanza para
  concluir.
- **El cálculo no daba la etiqueta.** «No concluyente» se armaba en la
  interfaz y en la memoria, cada una con su propia lógica.
- **Resultados que parecían completos.** Una falla sin la aportación de un
  motor se mostraba como número limpio.
- **Información de un resultado repartida.** Los datos usados, las hipótesis
  y lo que contribuía estaban en pasos de procedimiento distintos, sin una
  forma común de consultarlos.

## Criterio

`ieestudio_project::evaluation` define el estado y la trazabilidad una sola
vez. La interfaz, Revisar, la memoria y la línea de comandos los muestran
sin repetir la lógica.

| Estado | Significado | Ejemplo |
|---|---|---|
| **Válido** | Calculado con información suficiente y bajo hipótesis identificadas. | Falla de la acometida con potencia e impedancia; todos los interruptores la alcanzan. |
| **No concluyente** | Hay resultado, pero faltan datos que pueden modificarlo, o la cota conservadora no alcanza para concluir. | La falla no incluye un motor sin potencia; un interruptor menor que la cota superior. |
| **Sin evaluar** | No hay condiciones para calcular: falta un dato. | Transformador sin impedancia; interruptor sin capacidad interruptiva. |
| **Dato inválido** | Los datos son inconsistentes. | Impedancia de 0 %, potencia negativa, una conexión que el secundario no tiene. |

**El estado no dice si algo cumple.** Dice qué tan completo es el resultado.
Un resultado válido puede cumplir o no cumplir, y solo entonces la etiqueta
es «Cumple» o «No cumple». En los otros tres casos la etiqueta es el estado.

**Cada nivel toma el estado de su parte más débil** (`Evaluation::worst`):
- Una barra vale lo que su falla.
- La verificación de un lugar vale lo que su falla y la comparación con sus
  interruptores juntas.

**Trazabilidad.** Cada falla de barras y cada verificación por lugar lleva su
`Trace`:
- qué se calculó y en qué barra;
- los datos usados (potencia, impedancia, tensiones, conexión, dato de la
  suministradora);
- las hipótesis (bus infinito o dato de la suministradora, tolerancia, sin
  impedancia de alimentadores, el método de motores de la 0028);
- lo que contribuye (la fuente y cada motor, o lo que llega a través de cada
  transformador);
- lo que falta;
- los problemas.

**Dónde se ve.**
- Interfaz: «Cómo se obtuvo» en la ficha del tablero y en cada tarjeta de
  Revisar.
- Memoria: la misma lista en la sección de capacidad interruptiva.
- Línea de comandos: imprime lo que falta y los problemas.
- Resultado no válido: Revisar marca la falla «incompleta» en color de
  aviso, y el unifilar escribe «INCOMPLETA: FALTAN MOTORES».

**Veredicto.** Los cuatro estados se traducen a `Status::NotEvaluated` en
todo lo que no es válido. Así el proyecto nunca se declara conforme con algo
incompleto o inválido (decisión 0009).

## Alternativas consideradas

- **Agregar «No concluyente» e «Inválido» a `Status`.** Cambiaría el conteo y
  el veredicto de todos los tramos a la vez. Se dejó `Status` para el
  resultado normativo y `Evaluation` para la completitud. Así se pueden
  migrar los demás tramos uno por uno sin romper el veredicto.
- **Estados solo en la interfaz.** Era la situación anterior: cada pantalla
  decidía por su cuenta. Se descartó.

## Limitaciones

- **Tramos sin `Evaluation`.** Las verificaciones de conductor, protección,
  caída y tierra siguen con `Status` y su procedimiento. Un dato que no
  sirve ahí sigue apareciendo como «sin evaluar» y no como «dato inválido».
  Migrarlas es trabajo pendiente.
- **Inválido frente a faltante.** Se distingue revisando los datos antes de
  calcular, no por el texto del error.

## Consecuencias para otros módulos

- `InterruptingGroup` lleva `evaluation`, `trace` y `label()`, la única
  etiqueta.
- `FaultStudy` lleva `source_trace`.
- `PanelResult` lleva `fault_trace`.
- `FaultIssue` distingue `Invalid`.
- El JSON del resultado incluye todo lo anterior.
