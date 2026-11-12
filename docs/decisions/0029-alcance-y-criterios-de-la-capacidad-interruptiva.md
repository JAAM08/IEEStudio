# 0029. Alcance y criterios de la revisión de capacidad interruptiva (D-1 a D-8)

**Estado:** aceptada (12 de noviembre de 2026). El dueño del proyecto
respondió las decisiones D-1 a D-8 planteadas en
`../decisiones-pendientes-d1-d8.md` después de la auditoría de Fable. Aclara
la 0023 y la 0028; no las sustituye. Se implementa en la etapa 1 del plan
(`../plan-1.0.md`). La NOM-001-SEDE-2012 es la norma vigente que aplica el
perfil.

**Implementado:** D-1 con el factor del perfil (etapa 1A). La impedancia de
media bobina queda pendiente: necesita un campo nuevo y va con el formato 4.

## Alcance: revisión de seguridad, no estudio de cortocircuito

IEEStudio estima **una cota superior de la corriente de falla simétrica
inicial** solo para comparar la capacidad interruptiva de los interruptores
con ella (110-9, 110-10). No es un estudio de cortocircuito:

- No calcula la corriente pico (ip), la de interrupción (Ib) ni la mínima
  para la sensibilidad de las protecciones, ni coordinación.
- No revisa la relación X/R con la que se probó cada interruptor; cerca de
  transformadores grandes la capacidad real puede ser menor que la de placa.
- No descuenta la impedancia de los conductores: todas las barras de una
  zona ven la falla de su fuente.
- No usa el factor de tensión `c` ni la corrección `KT` de la IEC 60909-0:
  el margen viene de reducir la impedancia de placa con la tolerancia del
  perfil. Los dos métodos no se mezclan.

El estudio completo según la IEC 60909-0 (edición 3.0, 2026) es del
programa de cortocircuito de la suite (decisión 0018). La 0028 coincide con
el planteamiento de la edición 2016 en la aportación de motores a través de
un transformador; **no se declara basada en la edición 2026** mientras no se
coteje con su texto. La memoria y la línea de comandos dicen este alcance.

## Criterios

**D-1. Fase a neutro en secundarios de 240/120 V con derivación central.**
- Con la impedancia de media bobina que dé el fabricante, se calcula con
  ella (la forma de capturarla se define al implementarla, con su prueba).
- Con solo la impedancia total: falla de fase a neutro = falla entre líneas
  × **1.67**, el máximo teórico del método punto a punto de Bussmann (nota
  2: aproximadamente 1.5, entre 1.33 y 1.67). Se elige el máximo para que el
  valor siga siendo cota superior. Va en `profile.csv` con su referencia y
  el resultado dice que es una estimación de ese método.
- Es el valor en las terminales y se usa para toda la zona, que es
  conservador porque los conductores solo lo reducirían.
- El dato de la suministradora en un servicio de 240/120 V se toma como la
  corriente máxima disponible en ese punto y no se multiplica.
- Si no se puede aplicar ninguno de esos casos, «No concluyente».
- Ejemplo: 50 kVA, 3 %, 240 V: 7.72 kA entre líneas, **12.89 kA** de fase a
  neutro; con interruptores de 10 kA, «No concluyente».

**D-2. Dónde está el dato de la suministradora.**
- El dato dice dónde está: en baja tensión, en el punto de conexión, o en
  media tensión, en el primario del transformador de la acometida, con su
  tensión.
- En media tensión se convierte en la impedancia de la red,
  `Zred = S / (√3 · V · Icc)` en por unidad de la potencia del
  transformador, y se combina con la del transformador ya reducida por la
  tolerancia.
- **Combinación en cuadratura:** `Z = √(Zred² + Ztr²)`. Sin conocer X/R, la
  suma de magnitudes da una impedancia mayor que la real y una corriente
  menor; la cuadratura nunca es mayor que la impedancia real si las dos son
  resistivas-inductivas (`|a + b|² = |a|² + |b|² + 2|a||b|cos Δ` con
  `cos Δ ≥ 0`), así que conserva la cota superior.
  - Ejemplo: 1000 kVA, 5.75 % (5.175 %), 480 V, 5 kA en 13.2 kV:
    `Zred = 0.00875`, `Z = 0.05248`, **22.92 kA** (la suma de magnitudes daba
    19.88 kA).
- Faltan la potencia, la impedancia o la tensión del transformador:
  «Sin evaluar» con su causa.
- Un dato en baja tensión mayor que el bus infinito del transformador de la
  acometida: «Dato inválido».
- Un dato sin decir dónde está no se supone de baja tensión: queda
  «Sin evaluar» con un botón para indicarlo. Así abren los archivos
  anteriores que traen el dato.

**D-3. Cargas que parecen motor.**
- Mientras haya un aviso sin resolver en una zona, su capacidad
  interruptiva es «No concluyente».
- «Es otra carga» se guarda en el circuito, quita el aviso en Revisar, la
  memoria y la línea de comandos, y queda en la trazabilidad.
- «Cambiar a motor» conserva los datos; si faltan los del motor, su
  aportación queda incompleta como cualquier motor sin datos.
- La aportación sigue siendo `k · Ipc` con `k = 4` del perfil, declarado como
  criterio de ingeniería. Excluir motores enclavados y el modelo de máquina
  de la IEC quedan fuera de esta etapa.

**D-4. Secundario de un transformador intermedio.** Sigue por bus
infinito, que es el que decide. Un segundo escenario con la red de arriba
conocida queda para después: solo podría decidir si se calcula con R y X o
con la suma en cuadratura de D-2. La suma de magnitudes de la aportación de
motores a través del transformador (H-18) pasa también a cuadratura.

**D-5. Capacidad del transformador de la acometida.**
- El transformador se marca como propio o de la suministradora.
- **Propio:** se compara la demanda calculada en kVA con su capacidad
  nominal; si la excede, «No cumple». Sin margen genérico; uno de
  crecimiento sería un criterio de proyecto explícito.
- **De la suministradora o sin indicar:** fuera de alcance, «capacidad del
  transformador del suministrador, la confirma el suministrador». No deja el
  dictamen incompleto.

**D-6. Dictamen.** Un encabezado, en este orden de prioridad, con el conteo
de lo demás debajo:

1. «No conforme en lo evaluado»: alguna verificación no cumple.
2. «Datos inválidos: requiere corrección»: hay datos que impiden evaluar.
3. «Evaluación incompleta»: faltan datos o hay resultados no concluyentes.
4. «Conforme en lo evaluado»: todo lo aplicable se evaluó y cumple.

Ejemplo: «No conforme en lo evaluado · además 2 datos inválidos y 3 sin
evaluar». Nunca dice «Conforme con NOM-001-SEDE-2012» sin «en lo evaluado».
«Pendiente» queda solo para la aprobación profesional. Igual en la
interfaz, la memoria, la línea de comandos y el servidor MCP.

**D-7. Formato 4.** Con el primer campo nuevo (D-3) el formato pasa a 4.
Se leen los formatos 1 a 4 sin perder ni suponer datos; se escribe el 4; una
versión anterior dice que el archivo es de una versión más nueva. Los
archivos que ya escribió la versión del PR #11 siguen diciendo 3.

**D-8. Tablas de impedancia.**
- La tabla de la NMX-J-116-ANCE que dio el dueño confirma
  `nmx_j_116_impedance.csv` renglón por renglón, y su nota 1 dice que los
  valores incluyen las tolerancias (inciso 5.3.9): con impedancia de tabla
  no se reduce otra vez.
- Su nota 3 (un monofásico tipo subestación toma la impedancia del
  trifásico tipo subestación) se agrega al perfil.
- La tolerancia del 10 % sobre la impedancia de placa sigue siendo criterio
  del perfil hasta tener el inciso 5.3.9.
- Tipo pedestal (NMX-J-285) y transformadores de potencia arriba de 500 kVA
  (NMX-J-284) no tienen tabla: piden la impedancia de placa hasta contar con
  esas normas.

## Consecuencias

- Campos nuevos: dónde está el dato de la suministradora, transformador de
  la acometida propio o de la suministradora, «es otra carga» en el
  circuito; después, impedancia de media bobina.
- Parámetros nuevos en `profile.csv`: factor de fase a neutro
  (`fault_line_to_neutral_factor`, 1.67) con su regla `line_to_neutral_fault`
  en `references.csv`.
- Las pruebas de `crates/ieestudio-project/tests/auditoria_pr11.rs` y
  `crates/ieestudio-cli/tests/auditoria_salidas.rs` se vuelven obligatorias
  conforme se corrige cada hallazgo, con el valor exacto de este documento.
