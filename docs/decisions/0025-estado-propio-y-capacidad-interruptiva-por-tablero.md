# 0025. El estado de un circuito es el de su cálculo; la capacidad interruptiva se revisa por tablero

**Estado:** aceptada (noviembre de 2026).

## Contexto

Con la falla estimada por bus infinito (decisión 0023), bastaba capturar los
kA de un tablero para que casi todos sus circuitos pasaran a «Sin evaluar»
por el mismo motivo. El pie decía «0 cumplen» aunque cable, interruptor y
caída estuvieran bien, Revisar repetía el mismo aviso once veces y el único
problema real del proyecto se perdía. Además, el paso Revisar mostraba un
número de pendientes, su lista otro y la memoria otro.

## Decisión

- **Dos preguntas separadas.** El estado de un circuito o alimentador
  responde solo a su cálculo propio: carga, conductor, protección, caída,
  tierra y tubería. La capacidad interruptiva de su interruptor se guarda
  aparte (`RunResult::interrupting`) y no cambia ese estado.
- **Una verificación por lugar.** Los interruptores instalados en un mismo
  lugar ven la misma corriente de falla, así que se revisan juntos: el punto
  de conexión (el interruptor principal) y las barras de cada tablero (sus
  circuitos y los alimentadores de sus subtableros). El resultado es un
  `InterruptingGroup` por lugar, que cumple solo si todos sus interruptores
  alcanzan la falla.
- **El dictamen no se relaja.** Cada lugar sin resolver cuenta como una
  verificación pendiente y el proyecto no se marca conforme mientras exista.
  Lo que cambia es que se cuenta una vez por lugar, no una por interruptor.
- **En la interfaz:** una franja en el paso Circuitos y una tarjeta en
  Revisar por lugar, con el campo de kA ahí mismo y la lista de
  interruptores que abarca. La ficha del circuito lo dice en un renglón.
- **Un solo conteo de pendientes.** La barra de pasos, la lista de Revisar y
  la memoria muestran el mismo número: verificaciones que no cumplen más
  pendientes, donde un tramo sin calcular cuenta una vez.
- **Dos palabras** para lo que no es cumple ni no cumple: «Sin evaluar»
  (falta un dato o la regla no cubre el caso) y «No concluyente» (se calculó
  con un método conservador y no alcanza para afirmar). «Pendiente» queda
  para la aprobación profesional.

## Consecuencias

- La memoria en PDF lleva la capacidad interruptiva por lugar, con sus
  interruptores, y la anota una vez en las observaciones.
- Quien lea el resultado por programa (`ieestudio calcular --json`) debe
  mirar `fault.groups` además del estado de cada tramo.
