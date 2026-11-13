# 0031. Falla en cada tablero descontando los conductores (punto a punto)

**Estado:** aceptada (12 de noviembre de 2026). El dueño del proyecto aprobó
los puntos P-1 a P-6 tal como están. Modifica la 0029 en un solo punto: «no
descuenta la impedancia de los conductores».

**Implementado en la etapa 1F.** Una precisión sobre P-2: la resistencia se
corrige a 20 °C con la fórmula de la Tabla 8, nota 2, de la NOM
(`R2 = R1 · (1 + α · (T2 − 75))`, α = 0.00323 en cobre y 0.00330 en
aluminio), la misma que ya usa la caída de tensión, en lugar de las
constantes 234.5 y 228.1 del artículo citado. Da 0.82235 en cobre contra
0.82229: la diferencia no se ve en ningún resultado y la fuente es la norma.
La temperatura de 20 °C está en `profile.csv`
(`fault_conductor_temperature_c`). El interruptor del secundario de un
transformador se compara con la falla de las barras de su tablero, como los
demás interruptores del tablero (P-5).

## Contexto

Hoy todas las barras de una zona ven la falla de su fuente: la de las
terminales del transformador o el dato de la suministradora. Es una cota
superior segura, pero en los tableros alejados pide interruptores de más
kA de los necesarios.

En obra se usa el método punto a punto: la falla baja en cada tablero
conforme se aleja de la fuente, porque cada alimentador agrega su
impedancia. El caso típico es un transformador tipo pedestal particular sin
el dato de CFE: con sus kVA y su impedancia de placa se obtiene la falla en
sus terminales y de ahí se baja tablero por tablero.

El programa ya tiene todo lo que hace falta. Los alimentadores llevan
calibre, material, longitud, conductores en paralelo y tipo de tubo, y el
perfil tiene la Tabla 9 de la NOM-001-SEDE-2012 (resistencia y reactancia
en Ω/km por tipo de tubo). **No se pide ningún dato nuevo.**

Este punto es parte de una revisión de seguridad de la capacidad
interruptiva (0029), no de un estudio de cortocircuito. El estudio completo
sigue siendo del programa de cortocircuito de la suite (0018).

## Propuesta

### P-1. Método: impedancias en serie, con la de la fuente en cuadratura

Para cada tablero:

1. **Impedancia de la fuente, en ohms.** Sale de la falla de la fuente que
   ya se calcula:
   - trifásico: `Zf = V / (√3 · Icc)`;
   - entre fases de un monofásico: `Zf = V / Icc`;
   - de fase a neutro en 1F-3H: `Zf = Vfn / Icc fn`, con la `Icc fn` de la
     etapa 1A.
2. **Impedancia de los conductores desde la fuente.** Se suman como números
   complejos, `R + jX`, tramo por tramo, porque de los conductores sí se
   conocen R y X:
   - por tramo: `R = r · L / n` y `X = x · L / n`, con `r` y `x` de la
     Tabla 9 según material, calibre y tubo, `L` en km y `n` conductores
     en paralelo;
   - lazo de una fase y neutro o de dos fases: dos conductores, `2 · Zc`;
   - trifásico: un conductor por fase.
3. **Combinación en cuadratura con la fuente:** `Z = √(Zf² + |Zc|²)`. De la
   fuente no se conoce la relación X/R. La cuadratura nunca da más
   impedancia que la real, por el mismo argumento de la 0029, D-2, así que
   la corriente sigue siendo una cota superior.
4. **Falla en el tablero:** `Icc = V / (√3 · Z)` en trifásico y `V / Z` en
   monofásico, más la aportación de los motores (P-3).

**Comparado con el método de Bussmann:** su punto a punto suma magnitudes
(`M = 1 / (1 + f)`), lo que equivale a `Zf + |Zc|`. Eso da más impedancia
que la cuadratura y, por lo tanto, menos corriente. La cuadratura queda
siempre arriba de Bussmann: es el lado seguro.

### P-2. Temperatura del conductor: 20 °C

- La Tabla 9 da la resistencia a 75 °C. Un conductor frío tiene menos
  resistencia y deja pasar más falla.
- Para la cota superior se corrige a 20 °C, como en la falla máxima de la
  IEC 60909-0:
  - cobre: `R20 = R75 · (234.5 + 20) / (234.5 + 75)`, factor 0.8223;
  - aluminio: `R20 = R75 · (228.1 + 20) / (228.1 + 75)`.
- Las constantes 234.5 y 228.1 son propiedades del material. Van en
  `profile.csv` con su fuente.
- La reactancia no cambia con la temperatura.

En el ejemplo, la temperatura pesa más que la forma de combinar: en TA,
calcular a 75 °C da 4.85 kA, contra 5.79 kA a 20 °C.

### P-3. Motores: su aportación sin reducir en cada tablero de la zona

- La aportación de los motores (`k · Ipc`, `k = 4`, de la 0029) se suma
  completa en cada tablero de la zona, sin descontar los conductores entre
  el motor y la falla.
- Es conservador y no cambia el criterio ya aprobado.
- Así lo hace el método rápido de los ejemplos publicados, en el que la
  aportación se suma al inicio.

### P-4. Un alimentador sin datos no se descuenta

- Si a un tramo le falta la longitud o un calibre calculado, ese tramo no
  reduce nada. El tablero conserva la falla del de arriba, que es una cota
  superior válida.
- No es «Sin evaluar»: se sigue concluyendo, con un aviso en el
  procedimiento: «sin descontar el alimentador de TA: falta su longitud».
- Nunca se supone una longitud.

### P-5. Dónde se aplica

- **Desde el transformador de la acometida** (o desde el dato de la
  suministradora): a cada tablero, por su cadena de alimentadores.
- **Desde un transformador intermedio:** a partir de su secundario, con los
  conductores del secundario y los alimentadores que siguen. Su secundario
  sigue partiendo del bus infinito de ese transformador; la red de arriba
  no se descuenta (0029, D-4).
- **Los interruptores** de un tablero se comparan con la falla de ese
  tablero. El interruptor del alimentador está en el tablero de arriba y se
  compara con la falla de ese tablero.

### P-6. Cómo se ve

| Dónde | Qué muestra |
|---|---|
| **Revisar** | Cada tablero con su propia Icc y si sus interruptores alcanzan. Nada nuevo que capturar |
| **Unifilar** | «Icc = 5.79 kA (PUNTO A PUNTO)» en cada tablero, decreciente por cada derivación |
| **Memoria** | Por tablero: impedancia de la fuente, R y X de cada tramo con su renglón de la Tabla 9 y su corrección a 20 °C, la cuadratura y el resultado |
| **Línea de comandos y servidor MCP** | El mismo número y el mismo texto |

## Ejemplo

Pedestal particular de 500 kVA, 13.2 kV a 220Y/127 V, impedancia de placa
4.5 %, reducida 10 % a 4.05 % (0023). Conductores de cobre en tubo de PVC.

- `In = 500 000 / (√3 · 220) = 1312.2 A`
- En terminales: `Icc = 1312.2 / 0.0405 = 32.40 kA`
- `Zf = 220 / (√3 · 32 399) = 3.920 mΩ`

| Tablero | Alimentador | Zc acumulada | **Propuesta** | A 75 °C | Bussmann (suma) | Hoy |
|---|---|---|---|---|---|---|
| TG | 10 m, 3 × 500 kcmil por fase | 0.49 mΩ | **32.15 kA** | 32.12 kA | 28.79 kA | 32.40 kA |
| TA (desde TG) | 40 m, 2 AWG | 21.59 mΩ | **5.79 kA** | 4.85 kA | 4.98 kA | 32.40 kA |
| TB (desde TA) | 25 m, 6 AWG | 54.76 mΩ | **2.31 kA** | 1.92 kA | 2.16 kA | 32.40 kA |

TA y TB pasan de necesitar interruptores de 35 kA a poder llevar los de
10 kA: es el ahorro que da el método. La propuesta queda siempre igual o
arriba de Bussmann y abajo del valor de hoy.

## Pruebas

- **Matemática:** el ejemplo de arriba, con cada paso.
- **De referencia:** los ejemplos publicados del método punto a punto, con
  sus mismos datos de entrada.
  - Nuestro resultado debe quedar igual o arriba del publicado, porque
    usamos la cuadratura y 20 °C.
  - Como sus constantes C salen de tablas IEEE y no de la Tabla 9, se
    compara la cota, no el número exacto.
  - Hacen falta casos firmados por un ingeniero para cerrar H-14.
- **Normativa:** cada tramo cita su renglón de la Tabla 9, y la corrección
  de temperatura cita su fuente.
- **De no regresión:**
  - un proyecto sin alimentadores capturados da lo mismo que hoy;
  - ningún tablero da más que su fuente;
  - ningún tablero da más que el de arriba.

## Lo que no cambia

- La falla en las terminales del transformador, el dato de la
  suministradora, el factor 1.67, la tolerancia de 10 %, la aportación de
  motores y los estados (0029, 0030).
- No se pide ningún dato nuevo.
- Sigue sin calcularse la corriente pico, la de interrupción ni la mínima,
  y sigue sin revisarse la X/R de prueba del interruptor.

## Referencias

- NOM-001-SEDE-2012, Tabla 9 (ya en el perfil).
- Eaton Bussmann, *Selecting Protective Devices* / *Electrical Plan Review*,
  método punto a punto.
- IEC 60909-0, temperatura del conductor de 20 °C para la falla máxima.
- CSE Magazine, «The Art of Protecting Electrical Systems, Part 12»:
  ejemplo resuelto y corrección de temperatura del cobre y el aluminio.
