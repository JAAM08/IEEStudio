# 0022. El unifilar con el formato de un plano eléctrico

**Estado:** aceptada (noviembre de 2026). Amplía la 0007 y resuelve su
limitación conocida.

## Contexto

El unifilar abreviaba todo («4-10, 1-10d», «EMT 16 mm · 12 m»), dibujaba
todas las cargas como una flecha y salía en los colores de la aplicación. El
dueño del proyecto pidió que fuera más profesional y apegado a la norma, con
la referencia de un curso que pide en el diagrama las características de la
acometida, de la subestación, de los alimentadores (conductores activos,
puesto a tierra y de puesta a tierra, longitud y corriente) y de las
protecciones (requisitos I.1 a I.4), con la simbología de la
NMX-J-136-ANCE-2019.

## Decisión

- **Dos modos.** *Revisión* colorea cada tramo según cumple, no cumple o sin
  evaluar; es el de la aplicación. *Plano* va en blanco y negro con la hoja
  completa; es el de la memoria, el PDF y el DXF. En el panel Unifilar se
  cambia entre los dos.
- **Bloque de cada tramo (I.3):** identificador y longitud; tubo en
  designación métrica con su tamaño en pulgadas; conductores de fase, neutro
  y de puesta a tierra de equipos por separado, con calibre, material y
  aislamiento; corriente nominal; caída del tramo y acumulada.
- **Interruptores (I.4):** polos y amperes (`3x25 A`) y las fases del
  circuito. El marco, el modelo, la marca y la capacidad interruptiva llegan
  con los datos de catálogo.
- **Acometida (I.1) y transformador (I.2):** tensión y fases-hilos de la
  acometida, medidor, y el transformador con kVA, tensiones e impedancia.
- **Tablero:** recuadro punteado alrededor de la barra y sus interruptores,
  con un encabezado de carga instalada y demandada en kVA y kW, factor de
  demanda, tensión y fases-hilos.
- **Símbolos por tipo de carga:** luminario, contacto, motor, salidas
  especiales (aire acondicionado, aparato fijo, equipo de cocina), carga
  general y tablero. Son los mismos de la interfaz (decisión 0021).
- **Subtableros abajo.** Un subtablero es una columna más del tablero que lo
  alimenta, con su interruptor, sus conductores y el símbolo de tablero; su
  dibujo cuelga debajo. Con varios, el de más a la derecha va primero para
  que las líneas no se crucen.
- **Filas.** Con más de doce columnas la barra sigue en una fila nueva, unida
  a la anterior por el lado izquierdo.
- **Hoja (modo plano):** simbología con solo los símbolos que aparecen,
  límite del tablero, notas de abreviaturas, cajetín con los datos de la obra
  y marco. En DXF el límite de los tableros va con línea discontinua
  (`DASHED`).

## Símbolos cotejados con la norma

Los símbolos se revisaron contra la NMX-J-136-ANCE-2007 (sección 4.2,
símbolos nacionales, y 4.3, unifilares de subestaciones):

| Símbolo | Figura |
|---|---|
| Interruptor termomagnético (automático): la línea se abre y un arco une los extremos | 4.2.14, 4.3.3 |
| Equipo de medición: cuadro con M | 4.3.10 |
| Transformador: dos devanados enfrentados (los dos círculos son del grupo generador) | 4.2.68, 4.3.6 |
| Tablero de distribución general: rectángulo con diagonal | 4.2.90 |
| Salida de lámpara incandescente: circunferencia con cuatro rayos | 4.2.52 |
| Receptáculo sencillo: circunferencia con dos diagonales | 4.2.27 |
| Motor: circunferencia con M | 4.2.67 |
| Salida para propósito especial: circunferencia con triángulo y letras (AA, AF, EC) | 4.2.60 |
| Acometida: línea con quiebre y flecha | 4.2.26 |

La carga general no tiene símbolo en la norma y se dibuja con una flecha.

## Pendiente

- La edición vigente es la NMX-J-136-ANCE-2019, que no se pudo consultar; el
  cotejo es contra la de 2007. Falta confirmar que estas figuras no cambiaron.
