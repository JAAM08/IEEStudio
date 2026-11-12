# 0028. Contribución de motores y propagación de la corriente de falla

**Estado:** aceptada (11 de noviembre de 2026). El dueño del proyecto eligió
la alternativa A entre las tres que se le presentaron. Corrige la decisión
0023 en tres puntos: la aportación de motores, el estudio sin datos de la
fuente y los transformadores con datos incompletos.

## Problema

- **Motores en todas las barras.** La 0023 sumaba 4 veces la corriente de
  **todos** los motores en **todas** las barras. Mezclaba amperes de 220 V
  con amperes de 480 V y sumaba motores que están detrás de un
  transformador.
- **Falla heredada de otra tensión.** Un transformador intermedio con datos
  que no servían dejaba a su secundario con la falla del primario, que está
  a otra tensión.
- **Corriente nominal equivocada.** La falla de un transformador usaba
  `S / (√3 · V)` siempre que el secundario fuera trifásico, aunque
  alimentara una o dos fases. Salía √3 veces menor (−42 %), del lado
  inseguro.
- **Estudio cancelado.** Sin dato de la acometida se cancelaba todo el
  estudio.
- **Motores sin datos descartados.** Un motor sin datos se descartaba sin
  aviso.

## Criterio

### Zonas de tensión

La instalación se divide en la zona de la fuente y la del secundario de cada
transformador intermedio. No se descuenta la impedancia de los alimentadores
(0023), así que todas las barras de una zona ven la misma falla.

- **Base en la zona de la fuente.** El dato de la suministradora o, si no se
  tiene, el bus infinito del transformador de la acometida.
- **Base en la zona de un transformador intermedio.** Su bus infinito. La
  red aguas arriba se toma sin impedancia, así que no depende de la fuente y
  se evalúa aunque la fuente no tenga datos.
- **Corriente nominal.** La misma con que el transformador se dimensiona:
  `S / V` si alimenta una o dos fases, y `S / (√3 · V)` si alimenta tres.

### Motores (alternativa A, la elegida)

Cada motor y compresor pertenece a la zona de su tablero.

- **Misma zona.** Aportan completos: `k · Ipc`, con `k = 4` del perfil
  (`fault_motor_multiplier`).
- **Zona aguas abajo.** Los motores de un secundario, con lo que a su vez les
  llega de más abajo, se modelan como una fuente detrás de su impedancia
  subtransitoria `Zm = Ib₂ / Im₂`, en por unidad de la base del
  transformador. Quedan en serie con la impedancia del transformador
  reducida por la tolerancia, `Zt`. Lo que llega al primario, en amperes a
  su tensión, es `Im = Ib₁ / (Zt + Ib₂ / Im₂)`.
- **Zona aguas arriba.** No se suman a un secundario calculado por bus
  infinito: la red infinita en el primario ya es la mayor aportación posible
  de lo que hay arriba, motores incluidos.

### Datos que faltan o no sirven

Se aplica la 0027:

- **Fuente sin ningún dato.** La capacidad interruptiva queda fuera de
  alcance (0018), pero solo en los lugares que dependen de la fuente, y el
  veredicto los nombra.
- **Dato que falta.** La zona queda sin evaluar, con la causa.
- **Dato inconsistente.** La zona queda como dato inválido, con el problema.
- **Nunca se hereda la falla de otra tensión.**
- **Motor sin datos.** Se lista con lo que le falta, y la verificación de
  cada barra a la que aportaría queda no concluyente.
- **Transformador sin datos con motores detrás.** Deja incompleta la falla de
  su primario.

## Alternativas presentadas

| | Método | Con 45 kVA, 3 %, 480/220 V y 112 A de motores en el secundario | Consecuencia |
|---|---|---|---|
| **A** (elegida) | A través de la impedancia del transformador | 50.05 A en 480 V | Modelo de la IEC 60909-0 y del Cuaderno técnico 158 de Schneider Electric. Usa un dato conocido del modelo. |
| B | Solo la relación de tensiones, `Im₂ · V₂ / V₁` | 51.33 A | Un poco más conservadora; el Cuaderno 158 la acepta como estimación. Ignora la impedancia del transformador. |
| C | Solo los motores de la misma zona | 0 A | Método punto a punto de Eaton Bussmann. Deja fuera una aportación real. |
| — | Sumar todo en todas las barras (0023) | 112 A sumados como si fueran de 480 V | Mezcla tensiones; se retira. |

## Justificación técnica

- **Qué exige la norma.** La NOM-001-SEDE-2012 pide que la capacidad
  interruptiva alcance la corriente de falla disponible (110-9, 110-10), pero
  no fija el método para calcularla.
- **IEC 60909-0.** Trata los motores asíncronos como fuentes detrás de
  `ZM = UrM² / (SrM · ILR/IrM)`. Con `ILR/IrM = 4` es la impedancia de esta
  decisión.
- **Referir a otra tensión.** Las impedancias se pasan al otro lado de cada
  transformador con el cuadrado de la relación.
- **Coincidencia entre referencias.** La IEC, el Cuaderno 158 y el método
  punto a punto coinciden en no sumar corrientes de motores de tensiones
  distintas.

Conviene distinguir dos cosas que no son lo mismo:

- **Validación numérica.** Que el programa calcule bien con el método
  elegido. La dan las pruebas y el cotejo independiente en ohms del informe
  de auditoría.
- **Validación normativa.** Que el método y los factores (`k = 4`,
  tolerancia del 10 %) sean los adecuados para la memoria de un proyecto
  real. La tiene que hacer un ingeniero con el texto vigente de las normas.

## Limitaciones

- **Impedancias omitidas.** No se descuenta la impedancia de los
  alimentadores ni la de la red aguas arriba de un transformador intermedio.
  Cada valor es una cota superior.
- **Motores lejanos.** Dentro de una zona se suman todos los motores, aunque
  uno lejano aporte menos.
- **Factor de los motores.** `k = 4` es un criterio de ingeniería del perfil,
  no un valor de la norma.
- **Solo la falla trifásica o de fase a fase.** En 120/240 V la falla de fase
  a neutro puede ser mayor. La falla a tierra, la relación X/R y la asimetría
  son del programa de cortocircuito de la suite.

## Consecuencias para otros módulos

- `BusFault` lleva su propia aportación de motores (`motors`, con `through`
  por transformador) y los motores sin datos (`incomplete`).
- `FaultStudy` ya no tiene una aportación global.
- La ficha, Revisar, el unifilar y la memoria muestran lo de cada barra.
