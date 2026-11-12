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
| `h01-acometida-240-120.iees` | H-01 | 25 kVA, 3 %, 240/120 V e interruptores de 5 kA: «Cumple» con 3.86 kA (falla entre líneas) y «Conforme» |
| `h01-intermedio-240-120.iees` | H-01 | Lo mismo en un transformador seco 480 → 240/120 V |
| `h02-dato-media-tension-5ka.iees` | H-02 | 5 kA sin tensión con subestación de 1000 kVA: se usan en 480 V, «Cumple» con 10 kA |
| `h02-dato-mayor-que-bus-infinito.iees` | H-02 | 50 kA con un bus infinito de 23.24 kA, sin aviso |
| `h03-bombas-como-otra-carga.iees` | H-03 | Dos «Bomba 50 hp» como otra carga: aviso de tipo y aun así «Cumple» con 9.8 kA |
| `h03-bombas-como-motor.iees` | H-03 | Las mismas como motor: 10.84 kA, «No concluyente» (correcto) |
| `h04-relacion-1-88.iees` | H-04 | 226/120 V: 2.365 kA en vez de 4.097 kA |
| `h05-conexion-contradictoria.iees` | H-05 | «Monofásico» en 220Y/127 V: 13.12 kA sin aviso |
| `h05-impedancia-30.iees` | H-05 | Impedancia de 30 %: 1.09 kA y «Cumple» sin aviso |
| `h06-acometida-sobrecargada.iees` | H-06 | 300 kW en 75 kVA: «Conforme» |
| `h07-todo-cumple.iees` | H-07 | Dictamen «Conforme con NOM-001-SEDE-2012» sin «en lo evaluado» |
| `h07-todo-cumple-tabla.iees` | H-07, H-09, H-10 | Impedancia de tabla: el plano dice «reducida 10 %»; la versión anterior no lo abre |
| `h09-transformador-sin-potencia.iees` | H-09, H-10 | La versión anterior no lo abre; TF-1 sin «Icc SIN ESTIMAR» en el plano |
| `h11-dato-invalido.iees` | H-10, H-11 | 0 kA: nota de «dato de la suministradora» en el plano y «dato inválido: sin evaluar» |
| `h12-motor-detras-de-transformador-sin-datos.iees` | H-12 | El lugar cuenta 1 motor sin datos; la lista del estudio, 0 |
| `h16-aislamiento-127.iees` | H-16 | Transformador 127 → 127 V sin aviso de relación 1:1 |
| `p17-secundario-con-otra-conexion.iees` | 17 | Elevador 127 → 240/120 V que rechaza un circuito de 240 V |
| `p19-longitud-cero.iees` | 19 | Circuito de 0 m que cumple |

Los hallazgos sin archivo propio: H-08 y H-17 son de la interfaz (se revisan
con capturas), H-13 y H-15 son textos (sus pruebas leen el texto), H-14 es la
falta de casos de referencia firmados y H-18 es documentación del método.
