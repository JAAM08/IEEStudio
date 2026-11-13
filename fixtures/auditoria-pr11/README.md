# Casos de las auditorías del PR #11

Un proyecto `.iees` por hallazgo de `docs/auditoria-fable-pr11.md` (H-xx) y de
`docs/auditoria-modelo-y-falla.md` (pendientes 17 a 20). Se abren en la
aplicación o con `ieestudio calcular <archivo>` para ver el hallazgo sin
programar. Los arma `crates/ieestudio-project/tests/casos/mod.rs`; si se
cambia uno ahí, se regeneran con

```
IEES_REGENERAR=1 cargo test -p ieestudio-project --test auditoria_pr11
```

`tools/auditoria.sh` corre las pruebas de cada hallazgo y dice cuáles siguen
abiertos. Los datos son los del informe de Fable; ninguno es un valor
normativo nuevo ni un caso de referencia firmado (`reviewed_by`).

| Archivo | Hallazgo | Qué muestra hoy |
|---|---|---|
| `h01-acometida-240-120.iees` | H-01 | 25 kVA, 3 %, 240/120 V e interruptores de 5 kA. Antes «Cumple» con 3.86 kA y «Conforme»; desde la etapa 1A, 6.44 kA de fase a neutro y «No concluyente» |
| `h01-intermedio-240-120.iees` | H-01 | Lo mismo en un transformador seco 480 → 240/120 V (corregido en la etapa 1A) |
| `h02-dato-media-tension-5ka.iees` | H-02 | 5 kA sin decir dónde, subestación de 1000 kVA. Antes se usaban en 480 V y «Cumple» con 10 kA; desde la etapa 1B, «Sin evaluar» hasta indicar dónde |
| `h02-dato-en-primario-5ka.iees` | H-02 | Los mismos 5 kA en el primario de 13.2 kV: 22.92 kA en 480 V, «No concluyente» con 10 kA (etapa 1B) |
| `h02-dato-mayor-que-bus-infinito.iees` | H-02 | 50 kA en baja tensión con un bus infinito de 23.24 kA. Antes sin aviso; desde la etapa 1B, «Dato inválido» |
| `h03-bombas-como-otra-carga.iees` | H-03 | Dos «Bomba 50 hp» como otra carga. Antes, aviso de tipo y aun así «Cumple» con 9.8 kA; desde la etapa 1C, «No concluyente» hasta confirmar |
| `h03-bombas-confirmadas-otra-carga.iees` | H-03 | Las mismas con «Es otra carga» confirmado: «Cumple» con 9.8 kA, sin aviso y con nota en la memoria (etapa 1C) |
| `h03-bombas-como-motor.iees` | H-03 | Las mismas como motor: 10.84 kA, «No concluyente» (correcto) |
| `h04-relacion-1-88.iees` | H-04 | 226/120 V sin topología declarada. Antes 2.365 kA como trifásico y «Cumple»; desde la etapa 1D, «Dato inválido» en la capacidad interruptiva y el alimentador sin evaluar hasta declarar la topología |
| `h05-conexion-contradictoria.iees` | H-05 | «Monofásico» en 220Y/127 V. Antes 13.12 kA sin aviso; desde la etapa 1D, «Dato inválido» |
| `h05-impedancia-30.iees` | H-05 | Impedancia de 30 %: 1.09 kA y «Cumple» sin aviso. Pendiente de la NMX-J-116-ANCE-2017 (decisión 0030, N-1) |
| `h06-acometida-sobrecargada.iees` | H-06 | 300 kW en 75 kVA: «Conforme» |
| `h07-todo-cumple.iees` | H-07 | Antes: dictamen «Conforme con NOM-001-SEDE-2012» sin «en lo evaluado»; desde la etapa 1E, «Conforme en lo evaluado» |
| `h07-todo-cumple-tabla.iees` | H-07, H-09, H-10 | Impedancia de tabla. Antes el plano decía «reducida 10 %»; desde la etapa 1E dice que la mínima de norma ya incluye la tolerancia |
| `h09-transformador-sin-potencia.iees` | H-09, H-10 | TF-1 sin potencia: desde la etapa 1E, «Icc SIN ESTIMAR (FALTA UN DATO)» en el plano y «Evaluación incompleta» |
| `h11-dato-invalido.iees` | H-10, H-11 | 0 kA. Antes, nota de «dato de la suministradora» en el plano y «dato inválido: sin evaluar»; desde la etapa 1E, «Datos inválidos: requiere corrección» e «Icc SIN ESTIMAR (DATO INVÁLIDO)» |
| `h12-motor-detras-de-transformador-sin-datos.iees` | H-12 | Antes el lugar contaba 1 motor sin datos y la lista del estudio 0; desde la etapa 2B se cuentan igual, uno por motor |
| `h16-aislamiento-127.iees` | H-16 | Transformador 127 → 127 V: desde la etapa 2B se marca como relación 1:1 |
| `p17-secundario-con-otra-conexion.iees` | 17 | Elevador 127 → 240/120 V que rechaza un circuito de 240 V |
| `p19-longitud-cero.iees` | 19 | Circuito de 0 m que cumple |
| `p31-punto-a-punto.iees` | 0031 | Ejemplo de la decisión 0031: 500 kVA, 4.5 %, 220Y/127 V. La falla baja tablero por tablero: 32.40 kA en terminales, 32.15 en TG, 5.79 en TA y 2.31 en TB (etapa 1F) |

Desde la etapa 1D todos están en el formato 5: cada sistema lleva su
topología declarada (salvo el de `h04`, que no la trae a propósito) y los
alimentadores trifásicos con neutro dicen «carga lineal».

Los hallazgos sin archivo propio: H-08 y H-17 son de la interfaz (se revisan
con capturas), H-13 y H-15 son textos (sus pruebas leen el texto), H-14 es la
falta de casos de referencia firmados y H-18 es documentación del método.
