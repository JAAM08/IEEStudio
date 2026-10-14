# Metodología de ingeniería

## Tres cosas distintas

El programa distingue tres niveles y nunca los mezcla:

| Nivel | Qué significa | Quién lo da |
| --- | --- | --- |
| **Cálculo matemático** | Las fórmulas se evaluaron con datos y tablas del perfil. Completo o incompleto | El programa |
| **Verificación normativa** | Lo calculado cumple el perfil normativo. Conforme solo si todo cumple y nada quedó sin evaluar | El programa |
| **Aprobación profesional** | El proyecto es adecuado para construirse | Quien revisa y firma; el programa nunca la da |

Cada verificación queda como **cumple**, **no cumple** o **no evaluado**. «No
evaluado» significa que falta un dato o que la regla aún no está implementada,
y basta uno para que la verificación normativa del proyecto quede pendiente.

## Criterios de cálculo

- **Valores sin redondear.** El motor calcula con precisión completa; solo la
  presentación redondea. Toda selección de tabla toma el valor inmediato
  superior.
- **Nada se supone.** Si una tabla no tiene el valor, el elemento queda «no
  evaluado» con la explicación.
- **El calibre del usuario se verifica, no se cambia.** El programa propone el
  menor calibre que cumple y dice qué criterio lo determinó.
- **Referencias.** Cada paso del procedimiento y cada verificación lleva la
  cláusula de la norma que aplica.

Los criterios de ingeniería concretos (caída de tensión, terminales, neutro,
tubería, límites) están en `decisions/0005-criterios-de-calculo.md`.

## Tres tipos de prueba

Cada módulo de cálculo tiene:

1. **Pruebas matemáticas** (`tests/math_cases.rs`): ecuaciones, unidades,
   linealidad, casos límite y datos inválidos. Comparan contra la física, no
   contra la norma.
2. **Pruebas de referencia** (`tests/reference_cases.rs` y
   `tests/engineering_cases.rs`): resultados calculados fuera del programa.
   Los casos de `fixtures/engineering-cases/` están en CSV para que un
   ingeniero los revise sin leer Rust; la columna `reviewed_by` registra
   quién los revisó. Un caso sin revisión se ejecuta, pero no cuenta como
   validado.
3. **Pruebas normativas** (`tests/normative_cases.rs` y las del perfil en
   `ieestudio-rules`): que la selección, los límites y las advertencias
   correspondan al perfil, que cada regla tenga su cláusula y que los
   parámetros correspondan a la edición.

Al agregar una regla de cálculo se agregan las tres.

## Ejemplo de por qué importan los casos por edición

Al armar los casos de referencia se esperaba 3 AWG de puesta a tierra para un
interruptor de 400 A, que es el valor del NEC. El motor dio 2 AWG. El texto de
la NOM-001-SEDE-2012 confirma 2 AWG (33.6 mm²) para 400 A: el motor tenía
razón y el caso estaba mal. Por eso los casos se verifican contra la edición
del perfil y no contra la memoria de quien los escribe.
