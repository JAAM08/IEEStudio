# Fuentes normativas

## Qué hay en el repositorio

Para cada perfil, en `data/<perfil>/`:

- **Valores numéricos de tablas** (ampacidades, impedancias, dimensiones,
  capacidades normalizadas), transcritos para que el programa calcule.
- **Identificadores de cláusulas** (`240-6(a)`, `Tabla 310-15(b)(16)`) en
  `references.csv`, con una descripción redactada para este proyecto.
- **Parámetros** (125 %, 100 A, 800 A, 1/0 AWG…) en `profile.csv`.

## Qué no hay

- El texto de ninguna norma. No se copian artículos, notas ni explicaciones;
  las descripciones de `references.csv` son propias.
- El PDF de la norma ni las hojas de cálculo de terceros usadas como
  referencia.

## Condiciones de uso

- La NOM-001-SEDE-2012 es una Norma Oficial Mexicana publicada por la
  Secretaría de Energía en el Diario Oficial de la Federación:
  https://www.dof.gob.mx/normasOficiales/4951/SENER/SENER.html
- Su contenido técnico se basa en gran parte en el National Electrical Code
  (NFPA 70), cuyo texto tiene derechos de autor de la NFPA.
- Por eso el repositorio se limita a valores numéricos, identificadores y
  texto original. Los valores numéricos de una tabla normalmente se consideran
  datos y no expresión protegida, pero **esto no es una opinión legal**. Antes
  de ampliar lo que se incluye (por ejemplo, notas o texto de artículos),
  debe confirmarse con alguien calificado.

La fuente con validez oficial es siempre la norma publicada. Los archivos de
`data/` son una transcripción de trabajo; si encuentras una diferencia,
repórtala (ver `SECURITY.md`).

## Perfiles

| Perfil | Norma | Edición | Datos |
| --- | --- | --- | --- |
| `nom-001-sede-2012` | NOM-001-SEDE-2012, Instalaciones Eléctricas (utilización) | 2012 | `data/nom-001-sede-2012/` |

## Otras fuentes de referencia (no incluidas)

- Hojas de cálculo del Ing. Juan Carlos Vega Mendoza: lógica de caída de
  tensión y tubería, y casos de prueba.
- Manual Técnico de Cables de Energía, Condumex, 2006: referencia teórica.
- Especificaciones CFE DCCSSUBT y DCCIAMBT: referencia para redes de
  distribución (fuera del alcance actual).
