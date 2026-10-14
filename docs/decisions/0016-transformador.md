# 0016. Transformadores dentro del proyecto

**Estado:** aceptada (octubre de 2026).

## Contexto

En obra industrial y comercial es común alimentar un tablero de 220Y/127 V
desde 480 V con un transformador seco. Hasta ahora el proyecto tenía una sola
tensión, la de la fuente, para todos los tableros.

## Decisión

- **Cada tablero tiene la tensión de sus barras:** la de la fuente o la del
  secundario del transformador más cercano aguas arriba. Sus circuitos, sus
  motores y sus subtableros se calculan con ella.
- **Un tablero puede alimentarse por un transformador** de 600 V o menos:
  potencia, impedancia, sistema del secundario, si lleva interruptor en el
  secundario y la longitud de los conductores del secundario. El alimentador
  es el primario; los conductores del secundario son un tramo más («SEC») con
  su procedimiento, sus verificaciones y sus decisiones de diseño.
- **Los dos lados se dimensionan con la capacidad del transformador:** la
  corriente de diseño no baja del 125 % de su corriente nominal, el
  porciento de la Tabla 450-3(b); así se puede usar el transformador
  completo. Se propone el interruptor normalizado igual o inmediato superior.
- **Protección (Tabla 450-3(b)).** Primario: 125 % (normalizado inmediato
  superior a partir de 9 A), 167 % o 300 % según la corriente si solo hay
  protección en el primario; 250 % si también la hay en el secundario.
  Secundario: 125 % (o 167 % debajo de 9 A). Sin interruptor en el secundario,
  sus conductores dependen de 240-21(c), que el programa no verifica: queda
  «no evaluado».
- **Capacidad.** La carga calculada del tablero (artículo 220, con sus
  subtableros) no debe pasar de la potencia del transformador.
- **Caída de tensión.** El secundario es un sistema derivado: la caída del
  secundario y de sus circuitos se cuenta desde el transformador.
- **Cortocircuito.** La impedancia hasta el primario se refiere al secundario
  con el cuadrado de la relación de tensiones y se suma la del transformador,
  `z · V² / S`, como reactancia pura (del lado de la corriente mayor). El
  interruptor del secundario se compara con la falla en las terminales del
  transformador; los circuitos, con la de las barras del tablero.

## Lo que no se hizo todavía

Transformadores de más de 600 V (Tabla 450-3(a)) y acometida en media
tensión, reglas de derivación de 240-21(c), conexión de neutro y puesta a
tierra del sistema derivado (250-30), corriente de energización y
autotransformadores (450-4). Un transformador con protección térmica
coordinada (notas 3 y 5) se revisa a mano.
