# Arquitectura

VoltForge es un workspace de Cargo con un crate por responsabilidad. La regla
principal: **el cálculo no depende de ninguna interfaz**. La aplicación de
escritorio, la línea de comandos, la memoria en PDF y los agentes de IA usan
las mismas funciones.

## Crates y dependencias

```
voltforge-core          tipos del dominio, unidades, errores, validación
   ↑
voltforge-rules         perfiles normativos: tablas, parámetros y referencias (data/)
   ↑
voltforge-electrical    fórmulas: corriente, caída, conductor, protección, tierra, tubería
   ↑
voltforge-project       modelo del proyecto, cuadro de cargas, procedimiento, cálculo rápido, veredicto
   ↑
voltforge-diagrams      diagrama unifilar como trazos y textos; exporta SVG
   ↑
voltforge-reports       memoria de cálculo en PDF (Typst)
   ↑
voltforge-cli           línea de comandos «voltforge»
voltforge-app           aplicación de escritorio (egui)
```

Una flecha significa «depende de». Ningún crate depende de uno de arriba.

| Crate | Depende de | No debe depender de |
| --- | --- | --- |
| `voltforge-core` | nada obligatorio (`serde` opcional) | tablas, fórmulas, interfaz |
| `voltforge-rules` | core | fórmulas, proyecto, interfaz |
| `voltforge-electrical` | core, rules | proyecto, interfaz |
| `voltforge-project` | core, rules, electrical | diagramas, reportes, interfaz |
| `voltforge-diagrams` | project | egui, Typst |
| `voltforge-reports` | project, diagrams, rules | egui |
| `voltforge-cli`, `voltforge-app` | todos los anteriores | — |

## Flujo de un cálculo

1. El usuario captura el proyecto (`voltforge_project::Project`), que se
   guarda como JSON con `format_version`.
2. `calculate` elige el perfil por su identificador (`Profile::by_id`) y
   calcula cada alimentador y circuito con las fórmulas de
   `voltforge-electrical`.
3. Cada tramo (`RunResult`) trae sus valores, sus verificaciones (`Check`),
   su procedimiento (`Step`) y las referencias normativas de ambos.
4. `ProjectResult.verdict` resume en tres niveles: cálculo, verificación
   normativa y aprobación profesional (ver `engineering-methodology.md`).
5. La interfaz, el unifilar, la memoria y la línea de comandos solo presentan
   ese resultado; ninguno vuelve a calcular.

## Reglas y referencias

La regla de cálculo y su referencia normativa son objetos separados:

- `Rule` (en `voltforge-rules`) identifica lo que aplica el motor, por ejemplo
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

`voltforge-app` usa egui. Tiene tres vistas: Proyecto (árbol, cuadro de
cargas, detalle y procedimiento), Unifilar y Cálculo rápido. Se prueba sin
pantalla con `xvfb-run` y la opción `--captura`.
