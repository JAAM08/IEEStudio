# 0011. Motores y motocompresores

**Estado:** aceptada (octubre de 2026).

## Contexto

Hasta la versión 0.1 un motor o un equipo de aire acondicionado se calculaba
como carga general y quedaba «no evaluado». Casi toda obra comercial o
industrial tiene motores, y sus reglas (artículos 430 y 440) cambian la
corriente, el conductor, el interruptor y el alimentador.

## Decisión

- **Corriente por tabla (430-6(a)(1)).** El usuario elige la potencia en hp de
  la Tabla 430-248 (una fase) o 430-250 (tres fases); la columna sale de la
  tensión del sistema según el texto de cada tabla (`motor_voltage_columns.csv`).
  La corriente de placa solo se pide para el relevador de sobrecarga.
- **Conductor (430-22, 440-32).** 125 % de la corriente, que debe cubrir tanto
  la ampacidad de terminales como la corregida. Las terminales se deciden por
  esa corriente de diseño y no por el interruptor, que en un motor puede ser
  2.5 veces su corriente: es lo conservador.
- **Interruptor del motor (430-52).** Se propone el máximo permitido: el
  porciento de la Tabla 430-52 para interruptor de tiempo inverso, con el
  valor normalizado inmediato superior (excepción 1), porque uno menor puede
  dispararse al arrancar. El usuario puede fijar otro: menor cumple; mayor,
  hasta el límite de la excepción 2, queda «no evaluado» con la nota de que
  solo se admite si el motor no arranca; arriba de ese límite no cumple.
- **El conductor del motor no se protege según 240-4** (Tabla 240-4(g)): lo
  protege el relevador de sobrecarga.
- **Sobrecarga (430-32(a)(1)).** Sin la corriente de placa, la verificación
  queda «no evaluada»: un motor sin ese dato no se declara conforme.
- **Motocompresores (440).** Con la ampacidad mínima y la protección máxima de
  la placa del equipo (440-35, 440-22(c)), se usan tal cual: es el dato que
  traen los minisplits. Con la corriente de carga nominal, 125 % para el
  conductor y 175 % para el interruptor redondeado hacia abajo, sin exigir
  menos de 15 A.
- **Cuadro de cargas.** La potencia de un motor o compresor es la de su
  corriente: V · I (· √3) en VA, por su factor de potencia en W. Es lo usual en
  las memorias y queda del lado conservador.
- **Alimentador (430-24, 440-33).** Los motores cuentan al 100 % y el mayor
  suma otro 25 %; el mayor se elige por potencia aparente y su corriente se
  lleva a la tensión del alimentador. La parte continua se calcula por VA, no
  por W, para que la corriente sea correcta con factores de potencia
  distintos.
- **Interruptor del alimentador.** Se propone el normalizado igual o inmediato
  superior a la corriente de diseño. Si el usuario fija uno mayor que lo que
  permite 240-4, se admite hasta el límite de 430-62 (el interruptor del
  motor mayor más el resto de la carga). Si el interruptor del alimentador es
  menor que el del motor mayor, se avisa: puede dispararse al arrancar.
- **Puesta a tierra (250-122(a)).** No se exige mayor que los conductores de
  fase. Pasa en motores, cuyo interruptor rebasa la ampacidad del conductor.

## Lo que no se hizo todavía

Fusibles e interruptores de disparo instantáneo (las otras columnas de la
Tabla 430-52 ya están en los datos), motores de varias velocidades o de
servicio intermitente (430-22(e)), motores de corriente continua y de 2300 V,
síncronos de factor de potencia unitario, factores de demanda de motores
(430-26), circuitos con enclavamiento (430-24 excepción 3) y acondicionadores
de aire de cuarto con cordón (440, parte G). Un caso así se puede capturar como
carga general o fijando el interruptor, y se revisa a mano.
