# Archivos para probar compatibilidad entre versiones

- `ef59295-*.iees`: escritos por la versión anterior al PR #11 (commit
  `ef59295`, PR #10) con `ieestudio ejemplo`. La prueba
  `los_archivos_anteriores_al_pr11_se_leen` comprueba que la versión actual
  los lee y los calcula. La línea de comandos de esa versión solo escribe el
  proyecto de ejemplo; los demás proyectos antiguos se agregan aquí con el
  prefijo del commit que los escribió.

## Archivos nuevos leídos con la versión anterior (H-09)

`tools/auditoria.sh anterior` compila la línea de comandos de `ef59295` e
intenta abrir cada caso de `fixtures/auditoria-pr11/`. Resultado del 11 de
noviembre de 2026 sobre `main` en `35cf49e`:

| Archivo | Versión anterior |
|---|---|
| `h09-transformador-sin-potencia.iees` | No abre: `missing field kva` |
| `h12-motor-detras-de-transformador-sin-datos.iees` | No abre: `missing field kva` |
| `h07-todo-cumple-tabla.iees` | No abre: `missing field impedance_percent` |
| `h05-conexion-contradictoria.iees` | No abre: `missing field impedance_percent` |
| Los otros 14 | Abren |

Es decir, H-09 alcanza más que lo que reportó Fable: además del
transformador intermedio sin potencia, una acometida con «impedancia de la
tabla» tampoco abre en la versión anterior, aunque el formato sigue diciendo
3.

## Desde la etapa 1B (formato 4)

Los archivos se escriben en el formato 4. Con `tools/auditoria.sh anterior`,
la versión `ef59295` rechaza los 19 casos: 15 con «el archivo usa el formato
4, más nuevo que el que este programa entiende (3)» y 4 con «missing field»,
porque esa versión lee todo el archivo antes de revisar el formato. Ninguno se
abre a medias. Desde la etapa 1B el programa revisa el formato antes de leer
lo demás, así que un archivo de un formato futuro siempre dará el mensaje
claro.
