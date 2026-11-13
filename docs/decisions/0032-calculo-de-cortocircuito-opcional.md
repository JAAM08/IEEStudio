# 0032. Cálculo de cortocircuito opcional por proyecto

**Estado:** aceptada (12 de noviembre de 2026). La pidió el dueño del
proyecto y eligió las dos opciones de abajo. Implementada en el PR del
cálculo de cortocircuito opcional, después de la etapa 1F.

## Decisión

- Cada proyecto puede apagar el cálculo de cortocircuito
  (`settings.fault_study`).
- **Viene encendido** en los proyectos nuevos y en los que ya existen. Solo
  se escribe en el archivo cuando está apagado. Por eso no cambia el
  formato: una versión anterior que no lo entiende calcula la falla, que es
  revisar más, no menos.
- **Apagado:**
  - no se estima ninguna corriente de falla ni se compara ningún
    interruptor;
  - el unifilar no lleva Icc;
  - la capacidad interruptiva queda **fuera de alcance, anotada** en
    Revisar, la memoria, la línea de comandos y el servidor MCP: «no se
    revisó en este proyecto porque el cálculo de cortocircuito está
    apagado; requiere estudio de cortocircuito».
- El dictamen puede ser «Conforme en lo evaluado» con esa nota, igual que
  un proyecto sin datos de la fuente (0018). Nunca dice que los
  interruptores cumplen.
- El aviso de las cargas que parecen motor deja de hablar de la falla:
  solo importa para su protección.
