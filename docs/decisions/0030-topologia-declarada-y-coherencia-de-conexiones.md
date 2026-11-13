# 0030. Topología declarada y coherencia de conexiones (H-04, H-05)

**Estado:** aceptada (12 de noviembre de 2026). El dueño del proyecto la
decidió al revisar la etapa 1D del plan (`../plan-1.0.md`). Aclara la 0026
(fuente única y topología) y la 0029; no las sustituye.

**Implementado en la etapa 1D:** T-1 a T-7, C-1 y C-2. N-1 a N-3 esperan la
tabla y la nota 3 de la NMX-J-116-ANCE-2017.

## Contexto

El programa deducía la topología del sistema (cuántas fases, si hay neutro,
si es estrella) de la relación entre la tensión entre fases y la de fase a
neutro, con tres criterios distintos (H-04). Un sistema de 226/120 V (1.88)
quedaba como monofásico de tres hilos para el modelo y como trifásico para
la falla. Además, el transformador de la acometida podía decir «Monofásico»
en un sistema trifásico y la falla se calculaba sin aviso con la tabla
monofásica (H-05).

## Matriz de decisiones

| Núm. | Tema | Decisión | Estado |
|---|---|---|---|
| T-1 | Topología | El sistema guarda su topología declarada; nada se deduce de la relación de tensiones | Aprobada, etapa 1D |
| T-2 | Relación de tensiones | Solo valida la topología declarada, con la tolerancia de `profile.csv` (1 %) | Aprobada, etapa 1D |
| T-3 | Relación incompatible | «Dato inválido» en lo que dependa de ella; nunca la configuración más cercana | Aprobada, etapa 1D |
| T-4 | Circuitos trifásicos | Según las fases declaradas, aunque la relación sea incompatible | Aprobada, etapa 1D |
| T-5 | Neutro portador, 310-15(b)(5) | Según la topología declarada y la carga no lineal capturada; sin datos, «Sin evaluar» | Aprobada, etapa 1D |
| T-6 | Carga no lineal | Sí / no / sin capturar; sin capturar no se supone lineal | Aprobada, etapa 1D |
| T-7 | Archivos anteriores | Toman la topología solo si sus tensiones son exactamente uno de los seis sistemas de la interfaz; formato 5 | Aprobada, etapa 1D |
| C-1 | Conexión del transformador de la acometida | Si contradice al sistema, «Dato inválido» en la capacidad interruptiva | Aprobada, etapa 1D |
| C-2 | Banco de tres monofásicos | Se captura como banco trifásico con la conexión del banco | Aprobada, etapa 1D |
| N-1 | Impedancia fuera del intervalo de la NMX-J-116 | Abajo del mínimo, aviso; arriba del máximo, «No concluyente» hasta confirmar la placa | **Pendiente**: tabla de la edición 2017 |
| N-2 | Nota 3: monofásico tipo subestación | Columna del trifásico tipo subestación de su clase de aislamiento, sin importar los kVA | **Pendiente**: tabla de la edición 2017 |
| N-3 | Renglón de un banco de monofásicos | Renglón monofásico con la potencia de cada unidad (kVA del banco / 3) | **Pendiente**: tabla de la edición 2017 |

## T-1 a T-7. Topología declarada

**T-1. Datos.** El sistema tiene, por separado:

- La topología, que fija el número de fases, el número de hilos y la conexión:
  - monofásico de dos hilos (1F-2H);
  - monofásico de tres hilos con derivación central (1F-3H);
  - trifásico de tres hilos en delta (3F-3H);
  - trifásico de cuatro hilos en estrella (3F-4H).
- La tensión entre fases.
- La tensión de fase a neutro.

Un sistema sin topología declarada no se clasifica. La delta de cuatro hilos
con derivación (fase alta) queda fuera de alcance.

**T-2. Validación.** La topología pide sus tensiones:

| Topología | Tensiones que pide | Relación entre fases y fase a neutro |
|---|---|---|
| 1F-2H | Una sola: de fase a neutro o entre fases | No se valida |
| 1F-3H | Las dos | 2, con la tolerancia del perfil |
| 3F-3H | Solo entre fases | No se valida |
| 3F-4H | Las dos | √3, con la tolerancia del perfil |

La tolerancia es `system_voltage_ratio_tolerance` en `profile.csv` (1 %,
criterio de ingeniería del perfil, no de la norma). Los seis sistemas de la
interfaz quedan dentro: 220/127 da 1.7323 y 480/277 da 1.7329, contra
1.7321 de √3.

**T-3. Relación incompatible.**

- Si la relación no corresponde a la topología declarada, o si no hay
  topología y la relación no corresponde a ninguna, el sistema es «Dato
  inválido».
- La capacidad interruptiva que se calcula con un transformador de ese
  sistema queda «Dato inválido», con las tensiones, la relación obtenida y
  las esperadas.
- No es «No cumple»: es un dato inconsistente, no una no conformidad
  demostrada.
- Lo que no depende de la relación sigue: la caída de tensión usa la
  tensión del circuito; la tabla de motores, la tensión del sistema.
- El dato de la suministradora en baja tensión no depende de la relación y
  se usa tal cual.
- **Ejemplo:** 226/120 V da «Dato inválido» en la capacidad interruptiva,
  nunca un 240/120 V supuesto.

**Sin topología declarada** (archivos anteriores con tensiones que no son de
la interfaz), lo que depende de ella queda «Sin evaluar»:

- la capacidad interruptiva calculada con un transformador;
- los circuitos de tres fases;
- el neutro portador de un circuito de dos fases.

Revisar ofrece declararla.

**T-4. Circuitos permitidos.**

- Un circuito de tres fases se admite si el sistema declara tres fases,
  aunque la relación sea incompatible. Ningún cálculo del circuito usa la
  relación.
- Uno de dos fases pide tensión entre fases.
- Uno de una fase pide tensión de fase a neutro.

**T-5. Neutro como conductor portador, 310-15(b)(5) de la NOM-001-SEDE-2012:**

| Circuito | Neutro portador | Inciso |
|---|---|---|
| Una fase y neutro | Siempre | — |
| Dos fases y neutro de un 3F-4H en estrella | Sí | 310-15(b)(5)(2) |
| Dos fases y neutro de un 1F-3H | No: solo lleva el desequilibrio | 310-15(b)(5)(1) |
| Tres fases y neutro de un 3F-4H en estrella | Sí con carga mayormente no lineal; no con carga lineal | 310-15(b)(5)(3) y (1) |
| Tres fases y neutro, carga no lineal sin capturar | **Sin evaluar** | — |
| Dos fases y neutro, sin topología declarada | **Sin evaluar** | — |

**T-6. Carga no lineal.**

- Es un dato de tres estados: sí, no o sin capturar.
- Sin capturar, el conductor de un circuito o alimentador trifásico con
  neutro en estrella queda «Sin evaluar» y dice que falta ese dato.
- En los archivos anteriores, «no» era el valor de fábrica y no se distingue
  de una decisión. Se lee como «sin capturar»; «sí» se conserva.

**T-7. Archivos anteriores y formato 5.**

- Un sistema de un archivo anterior toma la topología solo si sus tensiones
  son exactamente las de uno de los seis sistemas que ofrece la interfaz:
  1F-2H 127 V, 1F-3H 240/120 V, 3F-4H 220Y/127 V, 208Y/120 V y 480Y/277 V,
  y 3F-3H 440 V. Es la topología con que ese sistema se mostraba al
  elegirlo.
- Cualquier otro sistema queda sin declarar.
- El archivo pasa al formato 5, porque una versión anterior ignoraría la
  topología y la volvería a deducir de la relación.
- Rige igual para el motor, la importación de archivos y las entradas del
  servidor MCP. No hay otra clasificación.

## C-1 y C-2. Conexión del transformador de la acometida

**C-1.**

- Si la conexión del transformador de la acometida dice «Monofásico» y el
  sistema declara tres fases, o dice una conexión trifásica y el sistema
  declara una fase, la capacidad interruptiva que se calcula con ese
  transformador es «Dato inválido». El mensaje dice:

  > La configuración del transformador de la acometida no corresponde al
  > sistema declarado. Verifica si capturaste una unidad monofásica o un
  > banco: varias unidades monofásicas que forman un banco trifásico se
  > capturan con la conexión del banco completo.

- No se supone ninguna conexión ni se busca la impedancia monofásica para
  calcular como trifásico.
- Con el dato de la suministradora en baja tensión el transformador no
  interviene y no se valida.
- Los transformadores intermedios no usan su conexión en el cálculo; su
  coherencia es el pendiente 17 de la etapa 2.
- No es un umbral: no lleva parámetro en `profile.csv`.

**C-2.** Un banco de tres monofásicos que alimenta un sistema trifásico se
captura como banco trifásico, con su conexión real y la potencia del banco.

## N-1 a N-3. Pendientes de la NMX-J-116-ANCE-2017

La tabla de `nmx_j_116_impedance.csv` viene de la lámina que dio el dueño;
**no está cotejada con la edición vigente 2017**. Hasta tenerla no se
implementan:

- **N-1. Impedancia fuera del intervalo.** Se calcula con la impedancia
  capturada, sin sustituirla:
  - Abajo del mínimo: aviso de que la falla sale mayor.
  - Arriba del máximo: «No concluyente» hasta una confirmación que se guarda
    asociada al transformador y al valor verificado. Se pierde si cambia la
    impedancia o un dato que cambie el renglón, y aun confirmada conserva el
    aviso.
  - Con el dato de la suministradora en baja tensión no aplica.
- **N-2. Nota 3.** Un monofásico tipo subestación toma el intervalo de la
  columna trifásica tipo subestación de su clase de aislamiento, aunque sea
  de menos de 225 kVA. Sin interpolar ni buscar la potencia más cercana.
- **N-3. Banco de tres monofásicos.**
  - Una casilla persistente, «Banco trifásico formado por tres
    transformadores monofásicos», hace que la tabla use el renglón
    monofásico con la potencia de cada unidad (kVA del banco / 3).
  - Ejemplo: 150 kVA son tres unidades de 50 kVA, con mínimo de 1.50 % en
    lugar de 2.00 %.
  - El cálculo conserva la potencia del banco y su impedancia en porciento,
    sin dividirla ni multiplicarla.
  - Un banco abierto, de dos unidades, o uno con unidades distintas no usa la
    tabla: pide la impedancia de placa.

**Riesgo mientras tanto.** Hoy, un banco de monofásicos con impedancia de
tabla toma el renglón trifásico tipo poste (2.00 %) y su falla puede salir
alrededor de 25 % menor que la cota. Hasta cerrar N-3, un banco se captura
con la impedancia de placa de sus unidades.

Sus casos de prueba están en `crates/ieestudio-project/tests/auditoria_pr11.rs`
con `#[ignore = "pendiente NMX-J-116-ANCE-2017"]`.

## Consecuencias

- `ieestudio-core::System` tiene `topology`. `has_three_phases` y las
  conexiones permitidas salen de ella.
- Nuevo parámetro `system_voltage_ratio_tolerance` (0.01) en `profile.csv` y
  nueva regla `system_voltage_ratio` en `references.csv`.
- La opción de sobrescritura `nonlinear` pasa a sí / no / sin capturar.
- Formato 5.
