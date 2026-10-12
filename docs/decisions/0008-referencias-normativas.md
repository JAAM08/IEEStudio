# 0008. Reglas y referencias normativas separadas

**Estado:** aceptada (auditoría de octubre de 2026).

## Contexto

Las cláusulas de la norma estaban escritas en mensajes del código, y algunos
valores normativos (125 %, 100 A, 800 A, 1/0 AWG) eran constantes. Cambiar de
edición habría obligado a tocar fórmulas.

## Decisión

- Cada regla tiene un identificador fijo (`Rule`).
- Su referencia (`RuleReference`: código, título, edición, cláusula, URL) se
  lee de `data/<perfil>/references.csv`, y los datos de la norma y los
  parámetros de `profile.csv`.
- Los pasos del procedimiento y las verificaciones llevan sus referencias; la
  interfaz y la memoria las muestran.
- En el repositorio solo hay identificadores y descripciones propias, nunca
  texto de la norma (ver `../normative-sources.md`).

## Consecuencias

Una edición nueva es una carpeta de datos más sus casos de prueba. Una prueba
normativa verifica que cada regla tenga su cláusula en cada perfil.
