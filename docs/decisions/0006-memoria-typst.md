# 0006. Memoria en PDF con Typst

**Estado:** aceptada.

## Decisión

La memoria se escribe como texto Typst a partir del resultado ya calculado y
Typst, incluido en el programa, la compone en PDF tamaño carta. La memoria no
calcula nada: usa los mismos pasos (`Step`) que muestra la interfaz.

## Consecuencias

- El ejecutable creció de 16 a 55 MB. Se reducirá al preparar el instalador.
- Un formato DOCX editable queda como opción futura en `ieestudio-reports`.
