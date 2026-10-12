# 0007. Unifilar como trazos independientes del dibujo

**Estado:** aceptada.

## Decisión

`voltforge-diagrams` genera el unifilar como una lista de trazos y textos en
coordenadas propias. La interfaz los pinta con egui, la memoria los recibe
como SVG, y una exportación a DXF para AutoCAD usará la misma lista.
Instalación radial: cada tablero es una barra, sus circuitos cuelgan en
columnas y sus subtableros a la derecha.

## Limitación conocida

Todos los circuitos de un tablero van en una sola fila; con muchos circuitos
el diagrama se vuelve muy ancho.
