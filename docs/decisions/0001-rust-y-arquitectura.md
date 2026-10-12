# 0001. Rust y motor independiente de la interfaz

**Estado:** aceptada.

## Contexto

VoltForge modela una instalación eléctrica una sola vez y de ese modelo deriva
cálculos, cuadro de cargas, unifilar y memoria. Debe correr en Windows primero,
también en Linux, más adelante en navegador, y ser utilizable por agentes de IA.

## Decisión

- Todo el programa en Rust, en un workspace de Cargo con un crate por
  responsabilidad (ver `../architecture.md`).
- El motor de cálculo no depende de ninguna interfaz. Todo lo que hace la
  aplicación se puede hacer desde la línea de comandos, que entrega JSON.
- Interfaz, mensajes y documentación en español; identificadores en inglés.

## Consecuencias

- El cálculo se prueba sin ventana y se reutiliza desde la línea de comandos,
  la memoria, un futuro servidor MCP o una interfaz distinta.
- Un ejecutable sin dependencias externas para el usuario.
