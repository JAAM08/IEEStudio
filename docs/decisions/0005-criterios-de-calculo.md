# 0005. Criterios de cálculo

**Estado:** aceptada. Las cláusulas de cada criterio están en
`data/nom-001-sede-2012/references.csv`.

- **Caída de tensión:** `ΔV = k · I · L · (R · cos θ + X · sen θ) / n`, con
  k = √3 en trifásico y 2 en los demás, R y X de la Tabla 9. La fórmula exacta
  con fasores se calcula como comprobación.
- **Ajuste de resistencia por temperatura:** opcional y apagado por defecto;
  `R2 = R1 · [1 + α · (T2 − 75)]`, Tabla 8, nota 2.
- **Cargas continuas:** solo la parte continua lleva el 125 %.
- **Selección de conductor:** el programa propone el menor calibre que cumple
  y dice qué criterio lo determinó (ampacidad, protección, caída o mínimo del
  usuario). Un calibre fijado por el usuario se verifica, nunca se cambia.
- **Terminales:** 60 °C hasta 100 A y 75 °C arriba de 100 A, sin exceder la
  temperatura del aislamiento.
- **Interruptores:** capacidad normalizada igual o inmediata superior; se
  omiten 16, 32 y 63 A (serie IEC) salvo que se activen.
- **Protección del conductor:** se admite el valor normalizado inmediato
  superior a la ampacidad hasta 800 A, con los límites de 240-4(d).
- **Puesta a tierra:** Tabla 250-122, con incremento proporcional cuando el
  conductor de fase se sobredimensiona.
- **Neutro como portador de corriente:** cuenta en una fase y neutro, y en dos
  fases con neutro de un sistema en estrella; en trifásico de cuatro hilos solo
  con carga mayormente no lineal.
- **Paralelo:** solo desde 1/0 AWG.
- **Tubería:** 53, 31 y 40 % según el número de conductores; niple al 60 %
  como opción por tramo; con tres conductores y relación de diámetros entre
  2.8 y 3.2 se sube un tamaño por atascamiento.
- **Límites de caída:** 3 % en derivado, 2 % en alimentador y 5 % acumulado,
  configurables. El 2 % de alimentador es criterio propio, más estricto que la
  nota de la norma.
