# 0013. Cortocircuito y capacidad interruptiva

**Estado:** aceptada (octubre de 2026).

## Contexto

La NOM-001-SEDE-2012 exige que cada interruptor tenga capacidad interruptiva
suficiente para la corriente de falla disponible en sus terminales de línea
(110-9) y que las protecciones y la impedancia del circuito permitan despejar
la falla sin daño extenso (110-10). Sin esa verificación un proyecto no se
puede declarar conforme, aunque todo lo demás cumpla.

## Decisión

- **Fuente.** El proyecto lleva la corriente de cortocircuito disponible en el
  punto de conexión (dato de la compañía suministradora) o, si no se tiene,
  la potencia y la impedancia del transformador. Con el transformador se toma
  el primario como fuente infinita: `I = S / (√3 · V · Z)` en trifásico y
  `I = S / (V · Z)` en una fase. Es la corriente mayor posible.
- **Método de impedancias.** A la impedancia de la fuente se le suma la de
  cada alimentador hasta el tablero, con la resistencia y la reactancia de la
  Tabla 9 entre los conductores en paralelo. Todo se resuelve del lado de la
  corriente mayor:
  - la fuente se toma como reactancia pura, de modo que la resistencia de los
    conductores se suma en cuadratura y la impedancia total sale menor;
  - la resistencia se lleva de 75 °C a 25 °C con el coeficiente de la Tabla 8,
    nota 2: un conductor frío deja pasar más corriente;
  - en un sistema trifásico se calcula la falla trifásica con la impedancia de
    un conductor; en uno de una fase, la falla recorre la ida y la vuelta.
- **Motores.** Se suma la aportación de todos los motores y compresores del
  proyecto, por omisión 4 veces su corriente a plena carga, en cada tablero.
  Es una cota superior: un motor lejano aporta menos. El factor está en los
  criterios del proyecto.
- **Dónde se compara cada interruptor.** Los de los circuitos de un tablero,
  con la falla en las barras de ese tablero. El de un alimentador, con la del
  punto de donde sale: la fuente para el principal y el tablero padre para un
  subtablero.
- **Capacidad interruptiva capturada.** Cada tablero lleva la de sus
  interruptores (sus circuitos y los alimentadores que salen de él) y cada
  alimentador puede llevar la suya; si no, toma la del tablero de donde sale.
  Sin capacidad capturada la verificación queda «no evaluada» y dice cuánta
  corriente hay disponible.
- **Sin datos de la fuente.** Solo el alimentador principal lo señala, como
  «no evaluado». Así el proyecto no se declara conforme sin esta verificación
  y no se llena de avisos repetidos.

## Lo que no se hizo todavía

Falla de fase a tierra y de fase a neutro (en las terminales de un
transformador estrella puede superar ligeramente a la trifásica), relación X/R
de la fuente y factor de asimetría, impedancia del transformador dentro del
proyecto (artículo 450), clasificación en serie de interruptores, interruptores
limitadores de corriente, capacidad de corto circuito de tableros y equipos
(110-10) y la resistencia del conductor a la falla. Un caso que lo requiera se
revisa a mano.
