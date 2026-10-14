# Arquitectura

IEEStudio es un workspace de Cargo con un crate por responsabilidad. La regla
principal: **el cálculo no depende de ninguna interfaz**. La aplicación de
escritorio, la línea de comandos, la memoria en PDF y los agentes de IA usan
las mismas funciones.

## Crates y dependencias

```
ieestudio-core          tipos del dominio, unidades, errores, validación
   ↑
ieestudio-rules         perfiles normativos: tablas, parámetros y referencias (data/)
   ↑
ieestudio-electrical    fórmulas: corriente, caída, conductor, protección, tierra, tubería, cortocircuito
   ↑
ieestudio-project       modelo del proyecto, carga del artículo 220, cuadro de cargas, cortocircuito, procedimiento, cálculo rápido, veredicto
   ↑
ieestudio-diagrams      diagrama unifilar como trazos y textos; exporta SVG
   ↑
ieestudio-reports       memoria de cálculo en PDF (Typst)
ieestudio-exchange      cuadro de cargas de ida y vuelta con Excel
   ↑
ieestudio-cli           línea de comandos «ieestudio»
ieestudio-app           aplicación de escritorio (egui)
```

Una flecha significa «depende de». Ningún crate depende de uno de arriba.

| Crate | Depende de | No debe depender de |
| --- | --- | --- |
| `ieestudio-core` | nada obligatorio (`serde` opcional) | tablas, fórmulas, interfaz |
| `ieestudio-rules` | core | fórmulas, proyecto, interfaz |
| `ieestudio-electrical` | core, rules | proyecto, interfaz |
| `ieestudio-project` | core, rules, electrical | diagramas, reportes, interfaz |
| `ieestudio-diagrams` | project | egui, Typst |
| `ieestudio-reports` | project, diagrams, rules | egui |
| `ieestudio-exchange` | project, rules | egui, Typst |
| `ieestudio-cli`, `ieestudio-app` | todos los anteriores | — |

## Flujo de un cálculo

1. El usuario captura el proyecto (`ieestudio_project::Project`), que se
   guarda como JSON con `format_version`.
2. `calculate` elige el perfil por su identificador (`Profile::by_id`) y
   calcula cada alimentador y circuito con las fórmulas de
   `ieestudio-electrical`.
3. Cada tramo (`RunResult`) trae sus valores, sus verificaciones (`Check`),
   su procedimiento (`Step`) y las referencias normativas de ambos.
4. `ProjectResult.verdict` resume en tres niveles: cálculo, verificación
   normativa y aprobación profesional (ver `engineering-methodology.md`).
5. La interfaz, el unifilar, la memoria y la línea de comandos solo presentan
   ese resultado; ninguno vuelve a calcular.

## Reglas y referencias

La regla de cálculo y su referencia normativa son objetos separados:

- `Rule` (en `ieestudio-rules`) identifica lo que aplica el motor, por ejemplo
  `Rule::StandardRatings`. El identificador no cambia entre ediciones.
- `RuleReference` dice dónde lo establece la norma: código, título, edición,
  cláusula y URL de la fuente.
- La cláusula de cada regla en cada edición vive en
  `data/<perfil>/references.csv`; los factores y límites en `profile.csv`.

Para agregar otra edición (por ejemplo una NOM-001-SEDE posterior) se agrega
una carpeta en `data/`, se registra en `Profile::by_id` y se agregan sus casos
en `fixtures/engineering-cases/<perfil>/`. Las fórmulas no cambian salvo que
la regla misma cambie.

## Guardado

El proyecto es un archivo JSON legible: se revisa en GitHub, lo pueden editar
agentes de IA y no requiere migraciones de base de datos. Los cambios de
formato suben `format_version`, y un programa rechaza archivos de una versión
más nueva que la suya. Ver `decisions/0003-archivo-json.md`.

## Interfaz

`ieestudio-app` usa egui con paneles acoplables (egui_dock). Todos los
paneles leen el mismo `Doc` (`state.rs`): el proyecto, su resultado y la
selección; cambiar algo en un panel recalcula y los demás lo reflejan.

```
src/app.rs           menús, espacios de trabajo, barra de estado, preferencias guardadas
src/state.rs         proyecto abierto, selección y acciones de edición
src/workspace.rs     paneles y acomodos de cada espacio de trabajo
src/panels/          un archivo por panel: browser, schedule, properties,
                     procedure, diagram, report, quick
src/widgets.rs       piezas compartidas: estados, propiedades, procedimiento
src/theme.rs         paleta, tipografía e íconos
```

Se prueba sin pantalla con `xvfb-run` y `--captura imagen.png`; `--espacio`,
`--panel` y `--propiedades` eligen qué capturar.
