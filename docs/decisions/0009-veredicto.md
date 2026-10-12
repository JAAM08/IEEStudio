# 0009. Veredicto en tres niveles

**Estado:** aceptada (auditoría de octubre de 2026).

## Decisión

El resultado separa:

1. **Cálculo matemático:** completo o incompleto.
2. **Verificación normativa:** conforme solo si todo cumple y nada queda sin
   evaluar; no conforme si algo incumple; pendiente en los demás casos.
3. **Aprobación profesional:** siempre pendiente. El programa no la emite.

Se muestra en la barra inferior de la aplicación, en el resumen de la memoria
y de la hoja de cálculo rápido, y al final de la línea de comandos.

## Consecuencias

Un proyecto con un circuito de climatización (reglas aún no implementadas)
nunca aparece como conforme, aunque todo lo demás cumpla.
