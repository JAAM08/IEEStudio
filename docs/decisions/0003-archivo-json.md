# 0003. Proyecto en un archivo JSON

**Estado:** aceptada. Se reconsideró SQLite en la auditoría de octubre de 2026
y se mantuvo JSON.

## Decisión

El proyecto se guarda como un archivo JSON legible con `format_version`. Los
resultados no se guardan: se recalculan al abrir.

## Razones

- Se puede revisar y comparar en GitHub, línea por línea.
- Los agentes de IA lo leen y lo editan sin herramientas especiales.
- No requiere migraciones de base de datos.

## Cuándo reconsiderarlo

SQLite tendrá sentido para catálogos grandes de equipos o historial de
revisiones. Si llega, será un crate `voltforge-storage` aparte, sin cambiar
el motor.

## Consecuencias

Un programa rechaza archivos con un `format_version` mayor al suyo en lugar de
leerlos mal.
