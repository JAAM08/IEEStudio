# 0033. Conexión del secundario, datos de captura y estados de los tramos

**Estado:** aceptada (12 de noviembre de 2026). El dueño del proyecto eligió
las tres respuestas de abajo para los pendientes 17, 19 y 20 de la auditoría
de Opus (`../auditoria-modelo-y-falla.md`) y la etapa 2C del plan. Los
pendientes 17 y 19 están implementados en la etapa 2A/2C; el 20 queda
diferido al rediseño.

## Pendiente 17. Conexión del secundario de un transformador

### Decisión

- La conexión de las barras de un tablero alimentado por un transformador
  **sale de la topología declarada de su secundario** (decisión 0030). No
  se agrega ningún campo:

  | Topología del secundario | Conexión de las barras |
  |---|---|
  | 3F-4H o 3F-3H | Tres fases |
  | 1F-3H (por ejemplo 240/120 V) | Dos fases con neutro |
  | 1F-2H con neutro | Una fase |
  | 1F-2H entre fases | Dos fases sin neutro |

- **Validación:** la topología del secundario debe ser posible con el
  transformador. Un transformador monofásico, o uno alimentado con una o dos
  fases, no puede dar un secundario trifásico. Si no corresponde, el
  transformador queda como **«dato inválido»** con el motivo, por ejemplo
  «un transformador alimentado con dos fases no puede dar un secundario
  3F-4H». Nunca se corrige solo.
- Sin topología declarada del secundario se dice que falta: «falta declarar
  la topología del secundario del transformador para saber su conexión».
- Los proyectos anteriores abren igual: su secundario ya tiene la topología
  que se le declaró en el formato 5.

### Ejemplo

Elevador de 127 V a 240/120 V (1F-3H). Antes, el tablero del secundario
heredaba la conexión de una fase del primario y rechazaba un circuito de
240 V. Ahora sus barras son de dos fases con neutro: acepta el circuito de
240 V y lo calcula a 240 V. Caso: `fixtures/auditoria-pr11/p17-secundario-con-otra-conexion.iees`.

## Pendiente 19 y etapa 2C. Datos de captura

### Decisión

- **Un circuito o alimentador nuevo nace sin potencia ni longitud.** Antes
  nacía con 1000 W y 10 m, que aparentaban un cálculo completo.
  - Sin potencia: «Sin evaluar: falta la potencia de la carga».
  - Sin longitud: «Sin evaluar: falta la longitud del circuito» (o «del
    alimentador», o «de los conductores del secundario»).
- **Una longitud de 0 m o negativa es «Dato inválido».** Nunca se toma como
  un tramo sin caída de tensión ni sin impedancia.
- **Una potencia negativa es «Dato inválido».** Una de 0 W sigue siendo un
  dato.
- **Un motor o un aire acondicionado** toman la potencia de sus datos de
  placa: no necesitan la capturada.
- **La carga de un tablero nunca se suma a medias.** Si a un circuito le
  falta la potencia, el alimentador de su tablero y los de todos los
  tableros de arriba quedan «Sin evaluar: falta la potencia del circuito
  C-07 de TG-1». La carga del tablero se muestra como «sin conocer» en la
  interfaz, la memoria, el unifilar y la línea de comandos, y el desbalance
  y el balanceo de fases esperan a tenerla.
- **Los archivos anteriores conservan lo que tenían guardado**, aunque fuera
  el 1000 W o el 10 m que ponía el programa. Un 0 m guardado ahora se marca
  como dato inválido al calcular.
- El archivo pasa al **formato 6**: la potencia y la longitud se escriben
  solo si existen. La clave del navegador pasa a `ieestudio-proyecto-web-6`;
  la del formato 5 se sigue leyendo.

### Para más adelante

Si hace falta el caso «equipo en el mismo gabinete», será una **casilla
explícita «sin tramo»**, no un 0 en la longitud.

## Pendiente 20. Estados de conductor, protección, caída y tierra

### Decisión

**Diferido al rediseño de interfaz.**

### Motivo

- Lo que cambia el resultado ya está resuelto. Desde la etapa 1E un dato que
  falta y uno inválido se distinguen en cada tramo y en el dictamen (D-6),
  y ninguno puede dar «Cumple» ni «Conforme».
- Lo que queda es cómo se presenta cada verificación (conductor, protección,
  caída y tierra) con cuatro estados en vez de tres. Es un cambio de
  presentación que conviene decidir con la interfaz nueva, para no hacerlo
  dos veces.

## Pruebas

- 17: `p17_el_secundario_tiene_su_propia_conexion`,
  `p17_un_monofasico_no_da_un_secundario_trifasico`,
  `a_two_phase_transformer_cannot_give_a_three_phase_secondary` y
  `a_two_phase_transformer_uses_its_single_phase_current`; en la interfaz,
  `a_new_circuit_uses_the_phases_of_the_secondary`.
- 19:
  - `p19_un_circuito_de_cero_metros_no_cumple_solo`;
  - `p19_un_circuito_sin_potencia_ni_longitud_queda_sin_evaluar`;
  - `a_missing_power_leaves_every_feeder_upstream_unevaluated`;
  - `a_feeder_without_length_is_not_sized`;
  - `equipment_does_not_need_a_captured_power`;
  - `a_motor_without_data_or_power_is_not_sized`;
  - `p31_un_tramo_sin_longitud_no_se_descuenta`;
  - `format_6_keeps_saved_values_and_missing_ones`;
  - `a_power_can_be_zero_but_not_negative` y
    `new_circuits_and_feeders_start_blank`, en la interfaz.
