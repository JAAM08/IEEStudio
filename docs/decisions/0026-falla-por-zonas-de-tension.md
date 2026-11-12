# 0026. Un dato en su nivel, falla por zonas de tensión y motores según su ubicación

**Estado:** propuesta (noviembre de 2026), pendiente de la aceptación del
dueño del proyecto. Corrige la 0023 en la aportación de motores, en el
estudio sin datos de la fuente y en los transformadores con datos
incompletos. El diagnóstico completo está en `../auditoria-modelo-y-falla.md`.

## Contexto

La auditoría del modelo eléctrico encontró que la 0023 sumaba la aportación
de todos los motores del proyecto en todas las barras, aunque estuvieran a
otra tensión o detrás de un transformador; que un transformador con datos
inválidos dejaba a su secundario con la falla del primario; que sin dato de
la acometida se cancelaba todo el estudio; y que al activar un transformador
la interfaz creaba potencia, impedancia y tensiones que nadie capturó.

## Principio

Un dato eléctrico se captura **una sola vez**, en el elemento y el nivel de
la instalación al que pertenece, y los elementos que dependen de él lo
obtienen del modelo de conexiones. Un dato que falta queda vacío y el
resultado dice cuál falta; nunca se reemplaza por un valor que parezca
válido.

- **Transformador de la acometida** (`Source.substation`): el que alimenta
  toda la instalación, de CFE o propio. Su secundario es el sistema de
  suministro; no se repite en ningún tablero.
- **Transformador intermedio** (`Panel.transformer`): uno adicional dentro de
  la instalación. Su primario es el alimentador del tablero, a la tensión de
  donde sale; su secundario fija la tensión del tablero y de lo que cuelga de
  él (`connection::bus_system`, la única deducción del sistema de unas
  barras; el cálculo y el unifilar la usan).
- Potencia, impedancia y longitud del secundario son `None` hasta que se
  capturan. Sin potencia el transformador no se dimensiona y su falla no se
  estima; sin impedancia, la falla no se estima. Un motor nuevo no tiene
  potencia hasta que se elige.
- Si el secundario de un transformador intermedio tiene la misma tensión
  que su primario, o si el primero de la instalación tiene la misma potencia
  que el de la acometida, se avisa: puede ser el mismo equipo capturado dos
  veces. No se rechaza, porque un transformador de aislamiento existe.

## Decisión

### Zonas de tensión

La instalación se divide en zonas: la de la fuente y la del secundario de
cada transformador intermedio. Como no se descuenta la impedancia de los
alimentadores (0023), todas las barras de una zona ven la misma falla.

- **Base de la zona de la fuente.** El dato de la suministradora o, si no se
  tiene, el bus infinito del transformador de la acometida.
- **Base de la zona de un transformador intermedio.** Su bus infinito: la
  red aguas arriba se toma sin impedancia, así que no depende de la fuente y
  se evalúa aunque la fuente no tenga datos.
- **Corriente nominal del transformador.** La misma con que se dimensiona:
  según la conexión del tablero (`S / V` si alimenta una o dos fases,
  `S / (√3 · V)` si alimenta tres). Antes la falla usaba siempre `√3` con un
  secundario trifásico y salía √3 veces menor.

### Motores

Cada motor y compresor pertenece a la zona de su tablero.

- **Misma zona:** aportan completos, `k · Ipc`, con `k` del perfil
  (`fault_motor_multiplier`, 4), como en la 0023.
- **Zona aguas abajo** (detrás de un transformador que sale de esta zona):
  los motores de ese secundario, con lo que a su vez les llega, se modelan
  como una fuente detrás de su impedancia subtransitoria,
  `Zm = Ib₂ / Im₂` en por unidad de la base del transformador, en serie con
  la del transformador reducida por la tolerancia, `Zt`. Lo que llega a las
  barras del primario, en amperes a su tensión, es
  `Im = Ib₁ / (Zt + Ib₂ / Im₂)`, con `Ib₁` e `Ib₂` las corrientes nominales
  de cada lado.
- **Zona aguas arriba:** no se suman a un secundario calculado por bus
  infinito: la red infinita en el primario ya es la mayor aportación posible
  de todo lo que hay arriba, motores incluidos.

Es el modelo de la IEC 60909-0 (motores asíncronos como fuentes detrás de
`ZM = UrM² / (SrM · ILR/IrM)`, que con `ILR/IrM = 4` es la misma impedancia)
y el del Cuaderno técnico 158 de Schneider Electric. Las dos referencias y
el método punto a punto de Eaton Bussmann coinciden en que no se suman
corrientes de motores de tensiones distintas.

### Alternativas consideradas

- **Referir con la sola relación de tensiones** (`Im₂ · V₂ / V₁`, sin la
  impedancia del transformador). Más simple y un poco más conservadora; el
  Cuaderno 158 la acepta como estimación. Se descartó porque ignora un
  elemento conocido del modelo; con un transformador de 45 kVA y 112 A de
  motores la diferencia es de 51.3 A frente a 50.05 A.
- **Solo los motores de la misma zona** (punto a punto de Bussmann). Deja
  fuera una aportación real; se descartó.
- **Sumar todo en todas las barras** (0023). Mezcla tensiones; se retira.

### Datos incompletos y estudio por partes

- **Fuente sin ningún dato:** lo que depende de ella queda **fuera de
  alcance** (0018), pero solo en esos lugares; el veredicto los nombra. Las
  zonas de transformadores intermedios con datos se evalúan.
- **Dato capturado pero incompleto o inválido** (transformador sin potencia o
  sin impedancia, impedancia de tabla que no existe): la zona queda **sin
  evaluar** con la causa, que llega a la ficha, a Revisar, a la memoria y a la
  línea de comandos. Nunca hereda la falla de otra tensión.
- **Motor sin datos** (sin potencia, sin placa del compresor o con una
  conexión que su sistema no tiene): no cuenta como cero ni desaparece. Se
  lista con lo que le falta y la capacidad interruptiva de cada barra a la
  que aportaría queda **no concluyente** aunque los interruptores alcancen
  la falla estimada. Un transformador sin datos con motores detrás deja
  incompleta, por la misma razón, la falla de su primario.
- **Dos hipótesis, dos textos:** con el dato de la suministradora la falla
  se dice «con el dato de la suministradora, sin descontar los
  alimentadores»; con un transformador, «por bus infinito». Las dos son cota
  superior y así se dice.

### Revisión general

Las cargas capturadas como «Otra carga» cuyo nombre parece de un motor o de
un aire acondicionado las detecta el modelo (`review::kind_hints`) y llegan
al resultado del proyecto: Revisar, la memoria y la línea de comandos las
muestran.

## Límites

- Sigue sin descontarse la impedancia de los alimentadores ni la de la red
  aguas arriba de un transformador intermedio: cada valor es cota superior.
- Un secundario usa la conexión de su primario (0024): un transformador
  alimentado de fase a neutro no puede dar circuitos de dos fases. Falta un
  dato de conexión del secundario en el modelo.
- En un secundario 120/240 V la falla de fase a neutro puede superar a la de
  fase a fase; aquí solo se estima la trifásica o la de fase a fase.
- Las longitudes de 0 m y los circuitos que nacen con 1000 W y 10 m son el
  mismo problema que los transformadores inventados; se corrigen con el
  rediseño de la captura.

## Pruebas

`tests/project_cases.rs`: `motors_contribute_by_voltage_zone` (números a
mano: 4.3739 kA y 4.4859 kA en el secundario; 50.05 A a través del
transformador y 25.106 kA en el punto de conexión),
`a_partial_study_evaluates_what_has_data`,
`a_transformer_without_data_never_inherits_the_primary_fault`,
`a_motor_without_power_makes_the_fault_incomplete`,
`a_two_phase_transformer_uses_its_single_phase_current`,
`a_transformer_with_equal_voltages_is_flagged` y
`a_transformer_without_power_is_not_evaluated`; en `fault.rs`, la fórmula
de transferencia en sus dos límites (sin impedancia del transformador, la
relación de tensiones; con aportación infinita, el bus infinito visto desde
el primario).
