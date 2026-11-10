# 0021. Agregar un circuito eligiendo primero qué se conecta

**Estado:** aceptada (noviembre de 2026).

## Contexto

Los motores y el aire acondicionado se calculan con sus propias reglas
(decisión 0011), pero en la interfaz no se veían: «Agregar circuito» creaba
siempre una carga general de 1000 W, y para que fuera motor había que abrir
la ficha, encontrar el desplegable «Tipo de carga» y elegirlo. Quien no lo
sabía capturaba el motor en watts y el programa lo calculaba como carga
general, sin la corriente de tabla, sin el 125 % y sin la protección del
artículo 430, y sin avisar. El dueño del proyecto lo señaló al usar la vista
guiada.

## Decisión

- **«¿Qué vas a conectar?»** Todo botón o comando que agrega un circuito,
  en las dos vistas, abre primero una ventana con ocho tarjetas: Alumbrado,
  Contactos, Motor, Aire acondicionado, Aparato fijo, Equipo de cocina, Otra
  carga y Subtablero. Cada una lleva su símbolo, para qué es y los artículos
  que aplica (de `references.csv`, con su descripción al pasar el cursor).
  Se elige también con las teclas 1 a 8.
- **El circuito nace con los datos de su tipo.** Un motor trae su potencia en
  hp; un aire acondicionado, su placa vacía y, si el tablero tiene dos fases,
  conexión entre fases. Todos van a las fases menos cargadas del tablero. Se
  abre su ficha con la descripción lista para escribir.
- **La ficha pide primero los datos del equipo.** Para motores y aire
  acondicionado, la potencia deja de capturarse («Potencia equivalente») y un
  recuadro «Por ser motor» o «Por ser aire acondicionado» muestra las reglas
  propias del equipo con su resultado: son los tres primeros pasos del
  procedimiento, así que no hay un segundo cálculo. La conexión se elige con
  su tensión (`1F · 127 V`, `2F · 220 V`, `3F · 220 V`) y solo se ofrecen las
  que tiene el tablero; al cambiarla se proponen las fases menos cargadas.
- **Los símbolos de la simbología de planos** (luminario, contacto, motor,
  salidas especiales y tablero) marcan el tipo en la ventana, en la tabla de
  circuitos y en la ficha, para que la interfaz y el unifilar hablen igual.
- **Subtablero desde los circuitos sin duplicar nada.** La tarjeta crea el
  mismo tablero que «+ Subtablero» en el paso Tableros (misma función, mismo
  ID libre, colgado del tablero de origen) y lleva a su ficha en ese paso
  para capturar su alimentador. Debajo de la tabla de circuitos se listan los
  subtableros que alimenta el tablero; se leen de la estructura del proyecto,
  no de una lista aparte.
- **Revisar pregunta por el tipo.** Un circuito de «Otra carga» cuyo nombre
  parece de motor (bomba, extractor, ventilador…) o de aire acondicionado
  (minisplit, clima, refrigeración…) aparece arriba de los pendientes con un
  botón para cambiarlo. Es una ayuda de la interfaz, no una verificación: no
  cambia el veredicto. «Es otra carga» lo oculta mientras la app esté
  abierta.

## Consecuencias

- Las reglas de cálculo no cambian; cambia que se encuentren. El formato del
  archivo tampoco cambia.
- Agregar un circuito es un clic más, a cambio de que el tipo nunca quede por
  omisión.
