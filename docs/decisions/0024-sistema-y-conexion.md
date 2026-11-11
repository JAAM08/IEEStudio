# 0024. El sistema decide qué conexiones existen

**Estado:** aceptada (noviembre de 2026).

## Contexto

El dueño del proyecto eligió en el cálculo rápido el sistema `1F-3H 240/120 V`
y pudo elegir «3 fases» en Conexión. El programa calculó con √3 y 240 V y
mostró un resultado que parecía válido. El sistema ya dice cuántas fases
hay: preguntarlas otra vez sobra, y aceptar la combinación imposible es un
error de cálculo.

## Decisión

- **Una sola regla, en el motor** (`ieestudio-core`): `System::allows` y
  `System::kinds` dicen qué conexiones tiene un sistema. El monofásico de
  tres hilos se reconoce porque su tensión entre líneas es el doble de la de
  línea a neutro (en una estrella trifásica es √3 veces); un sistema sin
  neutro se toma como trifásico de tres hilos.

  | Sistema | Conexiones |
  |---|---|
  | Una fase y neutro (127 V) | 1 fase |
  | Monofásico de tres hilos (240/120 V) | 1 fase (120 V) o 2 fases (240 V) |
  | Trifásico con neutro (220Y/127, 208Y/120, 480Y/277 V) | 1, 2 o 3 fases |
  | Trifásico sin neutro (440 V) | 2 o 3 fases |

- **El cálculo rechaza lo imposible.** Un tramo con una conexión que su
  sistema no tiene, o con más fases que el tablero que lo alimenta, queda
  «sin evaluar» con el motivo. Nunca se muestra un número.
- **La interfaz solo ofrece lo que existe**, con su tensión (`1F · 120 V`,
  `2F · 240 V`), en el cálculo rápido, el alimentador de un tablero y la
  ficha del circuito. Si solo hay una conexión, se muestra y no se pregunta.
- **Fases A, B, C:** se eligen con botones excluyentes entre las formas
  válidas. Si la conexión ocupa todas las fases del tablero no se preguntan.
- **«Lleva neutro»** solo aparece si el sistema tiene neutro; el cálculo no
  cuenta un neutro en un sistema que no lo tiene.
- **Proyectos ya guardados** con una combinación imposible abren igual. El
  paso Proyecto y Revisar avisan cuántos tramos quedaron así y ofrecen
  «Corregir todas»: cada tramo pasa a la conexión más cercana que sí existe
  (la de más fases sin pasar de la pedida) y sus fases se acomodan. En el
  cálculo rápido, al cambiar de sistema la conexión se ajusta sola y se avisa.

## Límites

- Una delta de 240 V con derivación central (3F-4H 240/120 V, «pierna alta»)
  tiene la misma relación de tensiones que el monofásico de tres hilos y se
  tomaría como tal. No está entre los sistemas que el programa ofrece; si se
  agrega, el sistema necesitará un dato explícito de su número de fases.
- Un tablero de una fase se toma en la fase A; elegir de qué fase del
  tablero padre sale queda pendiente.
