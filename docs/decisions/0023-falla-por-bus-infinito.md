# 0023. Capacidad interruptiva con la falla estimada por bus infinito

**Estado:** aceptada (noviembre de 2026). Amplía la 0018: el estudio de
cortocircuito completo sigue en la suite; dentro de IEEStudio entra una
estimación conservadora. La 0028 corrige la aportación de motores (por zona
de tensión, no todos en todas las barras), el estudio sin datos de la fuente
(por partes) y los transformadores con datos incompletos (sin evaluar, nunca
la falla del primario).

## Contexto

El unifilar debe indicar la capacidad interruptiva de cada protección
(requisito I.4) y la norma exige que alcance la corriente de falla disponible
(110-9, 110-10). Con la 0018 el veredicto la dejaba siempre fuera de
alcance. El dueño del proyecto propuso el método de bus infinito, que su
curso muestra con un transformador de 112.5 kVA, 220 V y Z = 3 %: 9.84 kA.
En la misma lámina, el resultado de un programa de cortocircuito suma 0.97 kA
de aportación de la carga de motores: sin ella el bus infinito se queda corto.

## Decisión

- **Fuente.** El dato de la suministradora (`source.fault.available_ka`)
  manda. Si no se tiene, el transformador de la acometida
  (`source.substation`): `In = S / (√3 · V)` o `S / V` en una fase, e
  `Icc = In / Z`, con la red sin impedancia. Sin ninguno de los dos, la
  capacidad interruptiva queda fuera de alcance, como en la 0018.
- **Tolerancia.** La impedancia de placa se reduce 10 % antes de calcular
  (`fault_impedance_tolerance` en `profile.csv`): la real puede salir menor y
  la falla, mayor.
- **Motores.** Se suma 4 veces la corriente a plena carga de todos los
  motores y compresores del proyecto (`fault_motor_multiplier`), en todos los
  tableros. Los dos valores son criterios de ingeniería, no de la norma, y
  están en el perfil para poder cambiarlos.
- **Tableros.** Cada tablero toma la falla del transformador del proyecto más
  cercano aguas arriba (también por bus infinito) o la del punto de conexión.
  No se descuenta la impedancia de los alimentadores. Todo da una cota
  superior.
- **Comparación.** Los interruptores de los circuitos, con la falla de las
  barras de su tablero; el de un alimentador, con la de donde sale; el del
  secundario, con la del transformador. La capacidad interruptiva es la del
  interruptor o, si no la tiene, la del tablero donde está.
  - Alcanza: **cumple** «con un método conservador».
  - No alcanza: **sin evaluar**, «no concluyente», porque la falla real puede
    ser menor; requiere estudio de cortocircuito. Nunca «no cumple».
  - Sin capacidad capturada: sin evaluar, con la falla estimada.
- **Veredicto.** Con datos de la fuente la capacidad interruptiva sale de la
  lista de fuera de alcance; si todo cumple, el veredicto lo dice:
  «La capacidad interruptiva se verificó con un método conservador (bus
  infinito)».
- **A la vista.** La ficha del tablero y los datos de la acometida muestran la
  corriente de falla, de dónde sale y, si hay motores, un recuadro con su
  aportación (cuántos, `4 × (…) A`, y la lista al pasar el cursor) y el
  procedimiento. La memoria lleva una sección con los pasos, la lista de
  motores y la falla de cada tablero.
- **Datos de catálogo** (requisitos I.2 e I.4 del unifilar): de la acometida,
  suministradora, frecuencia y medidor; de la subestación y de los
  transformadores, conexión, montaje y marca; del tablero, barras, capacidad
  interruptiva, modelo y marca; de cada interruptor, marco, capacidad
  interruptiva, modelo, marca, ajuste y posición en el tablero. Ninguno cambia
  el cálculo de conductores ni de protecciones.
- **Posiciones.** El programa propone la posición de cada interruptor según
  sus fases (nones a la izquierda, pares a la derecha, renglones A, B, C) y
  respeta las que fija el usuario.
- **Archivos.** El formato sigue siendo el 3. Los archivos de la 0013
  recuperan su falla de la suministradora y sus capacidades interruptivas; su
  factor de motores se ignora porque ahora es del perfil.

## Fuera de esta decisión

Falla de fase a tierra o a neutro, impedancia de los alimentadores, relación
X/R y asimetría, interruptores en serie y limitadores de corriente: son del
programa de cortocircuito de la suite.

## Transformador sin dato de placa

Con «Sin dato de placa» la falla de la subestación se calcula con la
impedancia mínima de la Tabla 11 de la NMX-J-116-ANCE
(`nmx_j_116_impedance.csv`) para su clase (monofásico, trifásico tipo poste o
tipo subestación), su potencia y la tensión de su primario. Ese mínimo ya
incluye la tolerancia (nota 1 de la tabla), así que no se reduce otra vez. Un
transformador que la tabla no cubre, como el tipo pedestal (NMX-J-285), no
recibe un valor supuesto: se pide la impedancia de placa y, sin ella, la
capacidad interruptiva queda fuera de alcance.

La tabla se transcribió de la lámina de un curso que la reproduce; falta
cotejarla con el texto de la norma.
