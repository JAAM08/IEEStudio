# 0012. Carga de los alimentadores según el artículo 220

**Estado:** aceptada (octubre de 2026).

## Contexto

Hasta la versión 0.2 el alimentador de cada tablero llevaba un factor de
demanda que capturaba el usuario y que se aplicaba a toda la carga, motores
incluidos. La norma no funciona así: el artículo 220 fija una carga mínima de
alumbrado por metro cuadrado, una carga mínima por salida de contacto y
factores de demanda distintos para cada tipo de carga y de inmueble. Un factor
único no se puede verificar y, aplicado a los motores, contradice 430-24.

## Decisión

- **Tipo de inmueble en el proyecto.** Se elige de la Tabla 220-12 y decide la
  carga mínima de alumbrado y qué factores aplican. Sin él, un alimentador con
  alumbrado o contactos queda «no evaluado».
- **Áreas en cada tablero.** Cada tablero declara la superficie que atienden
  sus propios circuitos; las de sus subtableros se suman. Un área puede ser de
  otro tipo dentro del inmueble (pasillos, lugares de reunión, bodegas, con su
  propio valor de la tabla), salvo en vivienda, donde la tabla no lo permite y
  toda el área va con 33 VA/m². Si el alimentador lleva alumbrado y no hay
  área, queda «no evaluado»: no se puede verificar la mínima de 220-12.
- **Tipos de carga.** Alumbrado, contactos, aparato fijo, equipo de cocina,
  motor, aire acondicionado y general. Cada uno va a su grupo en el
  alimentador.
- **Carga de cada grupo.**
  - Alumbrado general: la capturada o la mínima de 220-12, la mayor, con la
    Tabla 220-42 de su tipo de inmueble. Lo que se agrega para llegar a la
    mínima se toma como continuo, salvo en vivienda.
  - Contactos fuera de vivienda: la capturada o, en bancos y oficinas, la de
    11 VA/m² (220-14(k)), con la Tabla 220-44. La norma permite también la
    220-42; se usa la 220-44, que da una carga igual o mayor.
  - Vivienda: los contactos de uso general van con el alumbrado (220-14(j)) y
    la suma lleva la Tabla 220-42.
  - Aparatos fijos: 75 % con cuatro o más en el mismo alimentador, solo en
    vivienda (220-53). Se cuenta un aparato por circuito.
  - Equipo de cocina comercial: Tabla 220-56 según el número de equipos (uno
    por circuito), sin bajar de la suma de los dos mayores.
  - Motores y aire acondicionado: 100 %; el 25 % del mayor se suma en la
    corriente de diseño (430-24, 440-33), como antes.
  - Otras cargas: 100 %.
- **Salidas de contacto.** Un circuito de contactos puede llevar el número de
  salidas; fuera de vivienda su carga no puede ser menor que 180 VA por salida
  (220-14(i)) o no cumple. En vivienda no aplica (220-14(j)).
- **Corriente del alimentador.** Sale de la potencia aparente calculada. La
  parte continua de cada grupo conserva la proporción que tenía en lo
  capturado, y el 125 % se aplica solo a ella.
- **Otros métodos.** El alimentador puede ir «sin factores» (todo al 100 %:
  por ejemplo áreas de hospital u hotel donde se usa todo el alumbrado a la
  vez, nota de la Tabla 220-42) o con un factor propio. Un factor propio menor
  que 1 se aplica, pero deja el alimentador «no evaluado»: el artículo 220 no
  lo respalda por sí mismo (se justifica con 220-87 o 430-26).
- **Archivos anteriores.** El factor 1 de los archivos de formato 1 y 2 pasa al
  método del artículo 220; uno distinto de 1 se conserva como factor propio.
- **Todos los valores en datos.** Tablas 220-12, 220-42, 220-44 y 220-56,
  sección 220-14(k) y los parámetros de 220-14(i) y 220-53 en
  `data/nom-001-sede-2012/`.

## Lo que no se hizo todavía

Estufas y secadoras de vivienda (220-54, 220-55), calefacción eléctrica y
cargas no coincidentes (220-51, 220-60), aparadores y rieles de alumbrado
(220-43), contactos múltiples de cuatro o más a 90 VA, los 1500 VA por circuito
de aparatos pequeños y de lavadora de vivienda (220-52), el neutro del
alimentador (220-61), los métodos opcionales de la Parte D y los factores de
demanda de motores (430-26). Una estufa o una secadora se captura como carga
general, al 100 %: queda del lado seguro. Los circuitos de aparatos pequeños
se capturan como contactos con su carga real, de al menos 1500 VA.
