# Decisiones D-1 a D-8: lo que Antonio debe aprobar antes de la etapa 1

11 de noviembre de 2026. Base: `main` en `35cf49e`. Cada nota dice qué está
en juego, las alternativas, qué cambia para quien usa el programa, un
ejemplo con números y dónde verificarlo. **Ninguna se ha implementado.** Las
recomendaciones son técnicas; la decisión es de Antonio.

> **Resueltas el 12 de noviembre de 2026: ver la decisión 0029.** Este
> documento queda como el análisis que llevó a ella. Antonio eligió que
> IEEStudio siga siendo una revisión de seguridad y no un estudio de
> cortocircuito, el factor de fase a neutro de 1.67 (no 1.5) para conservar
> la cota superior, el transformador de CFE fuera de alcance y un dictamen
> con cuatro encabezados. Correcciones a este texto: la falla con dato de
> media tensión se combina en cuadratura (22.92 kA en el ejemplo de D-2, no
> 19.88 kA); el ejemplo de D-5 son 301.6 kVA calculados, no 300 kW; las
> secciones citadas de la IEC 60909-0 son de la edición 2016 y falta
> cotejarlas con la 3.0 de 2026; la nota 1 de la tabla de la NMX-J-116
> confirma que sus valores ya incluyen la tolerancia (D-8).

Los números se calcularon con el método del programa (impedancia de placa
reducida 10 % por la tolerancia del perfil) y se pueden repetir con los
casos de `fixtures/auditoria-pr11/`.

**Sobre las fuentes (al 11 de noviembre).** Los textos de la NOM-001-SEDE-2012, la NMX-J-116-ANCE
y la IEC 60909-0 son de pago o están en el DOF y no los tuve completos; las
referencias dicen el número de sección para que se cotejen en el texto
vigente. Existe un anteproyecto de nueva NOM-001-SEDE (2025); conviene
confirmar que la de 2012 sigue siendo la que se aplica en los proyectos de
Antonio.

---

## D-1 · Falla de fase a neutro en secundarios de 240/120 V (H-01)

**El problema.** En un transformador monofásico con derivación central la
falla de una línea al neutro solo recorre media bobina. Esa media bobina
tiene menos de la mitad de la impedancia de la bobina completa, así que la
corriente de fase a neutro en las terminales sale **mayor** que la de línea
a línea. Hoy el programa calcula solo la de línea a línea y la llama «cota
superior».

**Referencia.** Método punto a punto de Bussmann (Eaton), *Electrical Plan
Review*, página del método, nota 2: en las terminales de un transformador
monofásico con derivación central, I(F-N) ≈ 1.5 × I(F-F); el factor es
aproximado y en teoría va de 1.33 a 1.67; supone 1.5 × %R y 1.2 × %X para la
media bobina, red infinita y la falla en las terminales. A cierta distancia,
según el calibre, la de fase a neutro vuelve a ser menor. **No es un valor
de la NOM**: es un criterio de ingeniería publicado por un fabricante. La
NOM (110-9, 110-10) solo pide que la capacidad interruptiva no sea menor que
la falla disponible.

**Ejemplos.** Interruptores residenciales típicos de 10 kA:

| Transformador | Falla F-F hoy | × 1.33 | × 1.5 | × 1.67 |
|---|---|---|---|---|
| 25 kVA, 3 % | 3.86 kA | 5.13 | 5.79 | 6.44 |
| 37.5 kVA, 3 % | 5.79 kA | 7.70 | 8.68 | 9.66 |
| 50 kVA, 3 % | 7.72 kA | 10.26 | **11.57** | 12.89 |
| 50 kVA, impedancia de tabla (1.5 %) | 13.89 kA | 18.47 | 20.83 | 23.19 |

Con 50 kVA y 10 kA, hoy dice «Cumple»; con el factor sería «No concluyente».

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) Factor del perfil | `profile.csv` con el factor (1.5) y su referencia; la falla de 1F-3H es F-F × factor | Da un número y permite «Cumple» con margen. El factor queda a la vista y se puede cambiar. Es una aproximación: si el fabricante da la impedancia de media bobina, sería mejor usarla |
| (b) No concluyente | Sin factor: todo 1F-3H con falla estimada queda «No concluyente» | Seguro, pero ningún proyecto de 240/120 V con capacidad interruptiva llega a «Conforme» |
| (c) Solo quitar «cota superior» | Sigue dando «Cumple» con la F-F | **No resuelve**: sigue dando conforme un interruptor que puede no alcanzar la falla |
| (d) Dato de media bobina | Pedir %R y %X de media bobina al fabricante | Exacto, pero casi nunca está en placa; serviría como opción sobre (a) |

**Recomendación técnica:** (b) de inmediato (es lo que pide el plan, 1A) y
(a) cuando Antonio confirme la fuente. La regla llevaría su `Rule`, su
referencia a Bussmann y sus tres pruebas.

---

## D-2 · ¿A qué tensión está el dato de la suministradora? (H-02)

**El problema.** «Falla disponible» manda sobre el transformador de la
acometida y se usa tal cual en baja tensión. Si el dato que da CFE es del
lado de media tensión, se usa un número equivocado; y si es mayor que el
bus infinito del propio transformador (imposible), no se avisa.

**Ejemplo.** Subestación propia de 1000 kVA, 5.75 %, 480 V (bus infinito
23.24 kA):

| Dato capturado | Si es de 480 V (hoy) | Si es de 13.2 kV |
|---|---|---|
| 5 kA | 5 kA, «Cumple» con 10 kA | 114 MVA de red → **19.88 kA** en 480 V |
| 10 kA | 10 kA | 228.6 MVA → 21.43 kA |
| 50 kA | 50 kA sin aviso (imposible) | — |

Cálculo con dato de media tensión: `Scc = √3 · 13.2 kV · I`;
`Zred = Sbase / Scc` en por unidad; `Icc = In / (Zred + Ztr)`. Es el
equivalente de red de la IEC 60909-0, sección 6.2 (impedancia de la red
`ZQ = c · UnQ / (√3 · I"kQ)`, referida al otro lado con el cuadrado de la
relación). El factor de tensión `c` (1.1 en media tensión, tabla 1 de la
IEC 60909-0) subiría el resultado un poco; usarlo o no es parte de esta
decisión.

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) Decir dónde | Junto al dato: «en baja tensión (punto de conexión)» o «en media tensión, primario» con su kV. En media tensión se pasa por el transformador. Si no se dice, «Sin evaluar» | Correcto en los dos casos; un campo más en la acometida. Con dato de MT hacen falta kVA, Z y kV del transformador; si falta uno, «Sin evaluar» |
| (b) Solo baja tensión | El dato siempre es del punto de conexión en baja tensión; se avisa si supera el bus infinito | Más simple, pero si alguien captura el de MT el resultado queda del lado inseguro sin aviso |

En ambas: un dato mayor que el bus infinito del transformador de la
acometida es «Dato inválido» con su causa.

**Recomendación técnica:** (a). **Pregunta para Antonio:** ¿cómo entrega
CFE el dato en sus proyectos (kA o MVA, y en qué punto)?

---

## D-3 · Cargas que parecen motor (H-03)

**El problema.** El programa ya avisa «parece motor», pero da «Cumple» sin
su aportación. Dos «Bomba 50 hp» como otra carga: 9.8 kA y «Cumple»; como
motor: 9.8 + 4 × 2 × 130 A = **10.84 kA**, «No concluyente» con 10 kA.
Además «Es otra carga» no se guarda en el archivo.

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) No concluyente hasta confirmar | La capacidad interruptiva de esa zona queda «No concluyente» mientras haya un aviso sin resolver; «Es otra carga» se guarda en el circuito y quita el aviso en Revisar, memoria y línea de comandos; «Cambiar a motor» conserva los datos | Seguro. Un extractor de baño o un ventilador obliga a un clic para confirmar |
| (b) Solo aviso | Como hoy, más guardar la confirmación | Rápido, pero un dictamen «Conforme» puede omitir una aportación que el propio programa sospecha |
| (c) Estimar la corriente con los watts | Suponer que es motor y calcular su aportación con P / (√3 · V · fp) | **No recomendado**: inventa datos (regla 4 del plan) |

**Recomendación técnica:** (a). Requiere un campo nuevo en el circuito
(ver D-7). Texto propuesto del botón: «Es otra carga» con la nota «no
aporta a la falla».

---

## D-4 · Falla en el secundario de un transformador intermedio

**El problema.** Hoy el secundario se calcula por bus infinito aunque se
conozca la falla de arriba. Es seguro pero puede exagerar mucho, y entonces
interruptores que sí alcanzan la falla salen «No concluyente».

**Ejemplos** (480 → 220 V, falla aguas arriba conocida):

| Transformador | Arriba | Bus infinito (hoy) | Con la red de arriba |
|---|---|---|---|
| 45 kVA, 3 % | 25 kA | 4.37 kA | 4.05 kA (−7 %) |
| 112.5 kVA, 3 % | 10 kA | 10.93 kA | 7.28 kA (−33 %) |
| 300 kVA, 4.5 % | 25 kA | 19.44 kA | 14.33 kA (−26 %) |
| 500 kVA, 5.75 % | 25 kA | 25.36 kA | 17.31 kA (−32 %) |

No pude reproducir el caso de Fable (72.9 kA frente a 9.49 kA) porque su
informe no da los datos; los de arriba son propios y se pueden cotejar.

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) Seguir con bus infinito | Sin cambio (decisión 0028) | Siempre cota superior; más «No concluyente» |
| (b) Propagar | Si la falla de arriba es válida, `Zred = In₁ / Icc arriba` y `Icc = In₂ / (Zred + Ztr)`; si no, bus infinito | Más cercano a lo real y sigue siendo cota superior **si** la falla de arriba lo es |

**Advertencia técnica (liga con H-18).** Sumar magnitudes de impedancia en
vez de complejas da un resultado ligeramente **menor**: con X/R de 10 y 3 la
diferencia es 0.6 %; con 15 y 2, 2 %. En (b) eso puede pasar de cota superior
a cota casi superior. Para no perder la garantía habría que sumar
impedancias complejas (necesita X/R de la red y del transformador) o tomar
`Zred` solo como reactancia. Esto cambia la decisión 0028.

**Recomendación técnica:** (b) con la suma que conserve la cota superior,
pero solo después de cerrar D-1 a D-3. No es urgente: (a) es seguro.

---

## D-5 · Capacidad del transformador de la acometida (H-06)

**El problema.** 300 kW en un transformador de acometida de 75 kVA (402 %)
quedan «Conforme» porque esa capacidad solo se verifica en los
intermedios.

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) Verificar si es propio | Un dato nuevo: «de CFE» o «propio». Si es propio, la misma verificación que el intermedio (carga calculada del artículo 220 contra kVA) | Correcto: un transformador propio es parte de la instalación. Si es de CFE, su capacidad la decide CFE |
| (b) Solo aviso | Avisa cuando la carga supera la capacidad, sin afectar el dictamen | Simple; no distingue responsabilidades |
| (c) No revisar | Como hoy | Deja pasar el caso del ejemplo |

**Referencia.** La verificación de los intermedios hoy cita la carga del
alimentador (artículo 220). Qué criterio de capacidad (100 % de la carga
calculada, o con margen) es de Antonio; el programa no debe inventar uno.

**Recomendación técnica:** (a), con la carga calculada al 100 % (el mismo
criterio que ya tiene el intermedio) mientras Antonio no indique otro.

---

## D-6 · Texto del dictamen (H-07)

**El problema.** Con falla estimada, el dictamen dice «Conforme con
NOM-001-SEDE-2012: todo cumple y nada quedó sin evaluar». El programa no
revisa toda la norma (puesta a tierra completa, canalizaciones especiales,
etc.).

**Propuesta:** siempre «Conforme en lo evaluado con NOM-001-SEDE-2012», en
la interfaz, la memoria, la línea de comandos y el servidor MCP. Es el
criterio que Antonio ya aprobó el 3 de noviembre para cuando la capacidad
interruptiva estaba fuera de alcance; solo se extiende al caso con falla
estimada. No hay alternativa razonable que recomendar en contra.

---

## D-7 · Formato del archivo `.iees` (H-09)

**El problema.** El formato sigue diciendo 3, pero la versión anterior no
abre archivos nuevos con un transformador sin potencia **o con una
acometida con impedancia de tabla** (comprobado, `fixtures/compatibilidad/`).
D-3 (y quizá D-2 y D-5) agregan campos nuevos.

**Alternativas.**

| | Qué hace | Consecuencia |
|---|---|---|
| (a) Formato 4 | Subir a 4; la versión actual lee 1 a 4; una anterior dice claramente «el archivo usa el formato 4, más nuevo» (ya lo hace) | Honesto: nadie abre un archivo a medias. Un usuario con la versión vieja tiene que actualizar |
| (b) Escribir vacíos legibles | Escribir los datos faltantes de modo que la versión anterior los lea | Frágil y obliga a escribir valores que no existen (contradice 0026) |

**Recomendación técnica:** (a), en el mismo PR que agregue el primer campo
nuevo. Se agrega la prueba de lectura de los archivos de
`fixtures/compatibilidad/`.

---

## D-8 · Tabla de impedancias de la NMX-J-116 y su tolerancia

**El problema.** La tabla de `data/nom-001-sede-2012/nmx_j_116_impedance.csv`
(mínimos y máximos por tipo, kVA y clase de tensión) se transcribió de una
lámina (decisión 0023) y el programa supone que el mínimo **ya incluye** la
tolerancia, por eso no la reduce otro 10 %. Si no la incluye, la falla con
impedancia de tabla sale hasta 10 % baja.

**Ejemplo.** 112.5 kVA tipo poste a 13.2 kV, 220 V: con el mínimo de 2.0 %
tal cual, 14.76 kA; si hubiera que reducirlo 10 % (1.8 %), 16.40 kA.

**Qué se necesita.** El texto de la NMX-J-116-ANCE vigente (la que citan los
fabricantes junto con la especificación CFE-K1000-01) para cotejar los
renglones y leer si el valor de la tabla es nominal o garantizado. No pude
consultarla: no es pública. Mientras tanto la tabla sigue marcada «por
cotejar» y el programa no debe decir que está validada.

---

## Resumen para decidir

| Decisión | Recomendación técnica | Bloquea |
|---|---|---|
| D-1 | (b) ya; (a) con la fuente confirmada | Etapa 1A |
| D-2 | (a), y responder cómo da CFE el dato | Etapa 1B |
| D-3 | (a) | Etapa 1C |
| D-4 | (a) por ahora; (b) después, con suma que conserve la cota | Nada urgente |
| D-5 | (a) con carga al 100 % | Etapa 1E |
| D-6 | Sí | Etapa 1E |
| D-7 | (a) formato 4 | Primer campo nuevo (D-3) |
| D-8 | Conseguir el texto de la norma | Cierre de la 1.0 |

## Fuentes

- Eaton/Bussmann, *Electrical Plan Review* (método punto a punto, nota 2):
  https://www.mikeholt.com/documents/calculations/formulas/BulletinEPR-1-contents.pdf
- IEC 60909-0, secciones 6.2 (red) y 6.3 (transformadores) y tabla 1
  (factor c); resumen de las fórmulas:
  https://mycableengineering.com/iec-60909-fault-calculations/
- NOM-001-SEDE-2012, 110-9 y 110-10 (DOF, 29 de noviembre de 2012):
  https://sidof.segob.gob.mx/notas/docFuente/5280609
- Anteproyecto de nueva NOM-001-SEDE (2025):
  https://ecija.com/wp-content/uploads/2025/04/Anteproyecto-de-la-NOM-001-SEDE-instalaciones-electricas.pdf
- Fichas de transformadores tipo poste que citan NMX-J-116-ANCE y
  CFE-K1000-01: https://www.prolec.energy/wp-content/uploads/2025/10/PGE_Tech-Sheet_Single_Phase_Pole-Type_RGB_SPA_0625_V1.pdf
