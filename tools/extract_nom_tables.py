#!/usr/bin/env python3
"""Extrae tablas de la NOM-001-SEDE-2012 a archivos CSV.

Uso:
    pdftotext -layout NOM-001-SEDE-2012.pdf nom.txt
    python3 tools/extract_nom_tables.py nom.txt data/nom-001-sede-2012

El texto proviene de la publicacion del DOF. Cada tabla se valida con
comprobaciones de consistencia (por ejemplo, area = pi/4 * d^2) y las
discrepancias se imprimen para revisarlas a mano. Las erratas conocidas del
texto fuente se corrigen de forma explicita en ERRATA y quedan anotadas en
la columna `note` del CSV.
"""
import csv, math, re, sys
from pathlib import Path

src, out = Path(sys.argv[1]), Path(sys.argv[2])
L = src.read_text(encoding="utf-8", errors="replace").splitlines()
out.mkdir(parents=True, exist_ok=True)
NUM = r"[\d.]+"
DASH = {"––", "-", "–", "—"}
problems = []

def find(pat, start=0):
    for i in range(start, len(L)):
        if re.search(pat, L[i]):
            return i
    raise SystemExit(f"no se encontro: {pat}")

def val(tok):
    return "" if tok in DASH else tok

def write(name, header, rows):
    with open(out / name, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(header)
        w.writerows(rows)
    print(f"{name}: {len(rows)} filas")

# ---------------------------------------------------------------- Tabla 9
a = find(r"^\s*Tabla 9\.- Resistencia y reactancia")
b = find(r"^\s*Notas:", a)
rows = []
for ln in L[a:b]:
    t = ln.split()
    if len(t) == 16 and re.fullmatch(NUM, t[0]):
        rows.append([t[1]] + [val(x) for x in t[2:10]])
assert len(rows) == 21, len(rows)
write("table_9_impedance.csv",
      ["size", "x_pvc_or_aluminum", "x_steel", "r_cu_pvc", "r_cu_aluminum",
       "r_cu_steel", "r_al_pvc", "r_al_aluminum", "r_al_steel"], rows)

# ---------------------------------------------------------------- Tabla 8
a = find(r"^\s*Tabla 8\.- Propiedades de los conductores")
b = find(r"^\s*Notas", a)
t8 = {}
for ln in L[a:b]:
    t = ln.split()
    if len(t) == 10 and re.fullmatch(r"[\d/]+", t[0]) and re.fullmatch(NUM, t[1]):
        size, area, kcmil, strands, _sd, diam, total = t[:7]
        if strands == "1":
            continue  # se usa el cableado clase B
        t8[size] = [size, area, val(kcmil), strands, diam, total]
        d, at = float(diam), float(total)
        if abs(at - math.pi / 4 * d * d) / at > 0.02:
            problems.append(f"Tabla 8 {size}: area {at} vs pi/4 d^2 {math.pi/4*d*d:.2f}")
order = list(t8)
write("table_8_conductor_properties.csv",
      ["size", "area_mm2", "kcmil", "strands", "bare_diameter_mm", "bare_area_mm2"],
      list(t8.values()))

# ------------------------------------------------------ Tabla 310-15(b)(16)
a = find(r"^\s*Tabla 310-15\(b\)\(16\)\.-")
b = find(r"^\s*\* Véase 310-15\(b\)\(2\)", a)
rows, pending = [], None
for ln in L[a:b]:
    t = ln.split()
    if pending is not None and len(t) == 1:
        rows.append([t[0]] + pending)
        pending = None
    elif len(t) == 8 and t[1] == "**":
        pending = [val(x) for x in t[2:]]
    elif len(t) == 8 and re.fullmatch(NUM, t[0]) and re.fullmatch(r"[\d/]+", t[1]):
        rows.append([t[1]] + [val(x) for x in t[2:]])
assert len(rows) == 30, len(rows)
write("table_310_15_b_16_ampacity.csv",
      ["size", "cu_60c", "cu_75c", "cu_90c", "al_60c", "al_75c", "al_90c"], rows)

# ---------------------------------------------------------------- Tabla 4
SECTIONS = [
    ("EMT", r"Artículo 358 – Tubo conduit no metálico \(EMT\)"),
    ("IMC", r"Artículo 342 – Tubo conduit metálico semipesado \(IMC\)"),
    ("RMC", r"Artículo 344 –Tubo conduit metálico pesado \(RMC\)"),
    ("PVC80", r"Artículo 352 – Tubo conduit rígido de PVC \(PVC\), Cédula 80"),
    ("PVC40", r"Artículos 352 y 353 – Tubo conduit rígido de PVC \(PVC\), Cédula 40"),
]
t4a = find(r"^\s*Tabla 4\.- Dimensiones y porcentaje disponible")
t4b = find(r"^\s*Tabla 5\.- Dimensiones de los conductores aislados", t4a)
rowre = re.compile(r"^\s*(\d+)\s+(\S+)\s+([\d.]+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$")
rows = []
for code, pat in SECTIONS:
    s = find(pat, t4a)
    n = 0
    for ln in L[s + 1:t4b]:
        if "Artículo" in ln:
            break
        m = rowre.match(ln)
        if m:
            metric, trade, idmm, total, a60, a53, a31, a40 = m.groups()
            rows.append([code, metric, trade, idmm, total, a60, a53, a31, a40])
            n += 1
            tot = float(total)
            calc = math.pi / 4 * float(idmm) ** 2
            if abs(tot - calc) / tot > 0.01:
                problems.append(f"Tabla 4 {code} {metric}: area {tot} vs pi/4 d^2 {calc:.0f}")
            for frac, v in ((0.6, a60), (0.53, a53), (0.31, a31), (0.4, a40)):
                if abs(float(v) - frac * tot) > 1.5:
                    problems.append(f"Tabla 4 {code} {metric}: {frac} -> {v} vs {frac*tot:.1f}")
    assert n >= 10, (code, n)
write("table_4_conduit.csv",
      ["type", "metric_designator", "trade_size", "internal_diameter_mm",
       "area_100_mm2", "area_60_mm2", "area_53_mm2", "area_31_mm2", "area_40_mm2"], rows)

# ---------------------------------------------------------------- Tabla 5
a = t4b
b = find(r"^\s*Tabla 5 A\.-", a)
trip = []
for ln in L[a:b]:
    t = ln.split()
    if len(t) >= 4 and re.fullmatch(NUM, t[-1]) and re.fullmatch(NUM, t[-2]) \
            and re.fullmatch(r"[\d/]+", t[-3]) and re.fullmatch(NUM, t[-4]):
        trip.append((t[-3], t[-2], t[-1]))

def run(start_size, start_diam, count=None, until=None, after=0):
    for i in range(after, len(trip)):
        if trip[i][0] == start_size and float(trip[i][1]) == float(start_diam):
            j = i
            res = []
            while j < len(trip):
                res.append(trip[j])
                if (count and len(res) == count) or (until and trip[j][0] == until):
                    return res, j + 1
                j += 1
    raise SystemExit(f"ancla no encontrada {start_size} {start_diam}")

# Erratas del texto fuente: (grupo, calibre) -> (area correcta, explicacion)
ERRATA = {
    ("TW_THW_THHW_THW2", "10"): ("15.68", "el DOF imprime 55.68; pi/4 x 4.470^2 = 15.69 y el NEC, tabla de origen, indica 15.68"),
}
groups = {}
g, nxt = run("14", "4.902", until="2000")
groups["RHH_RHW_RHW2"] = g
small, nxt2 = run("14", "3.378", count=4, after=nxt)
big, nxt3 = run("6", "7.722", until="2000", after=nxt2)
groups["TW_THW_THHW_THW2"] = small + big
g, nxt4 = run("14", "2.819", until="1000", after=nxt3)
groups["THHN_THWN_THWN2"] = g
g, _ = run("14", "3.378", until="2000", after=nxt4)
groups["XHHW_XHHW2_XHH"] = g
rows = []
for name, items in groups.items():
    for size, diam, area in items:
        note = ""
        if (name, size) in ERRATA:
            area, note = ERRATA[(name, size)]
        calc = math.pi / 4 * float(diam) ** 2
        if abs(float(area) - calc) / calc > 0.015:
            problems.append(f"Tabla 5 {name} {size}: area {area} vs pi/4 d^2 {calc:.2f}")
        rows.append([name, size, diam, area, note])
write("table_5_insulated_conductors.csv",
      ["insulation_group", "size", "diameter_mm", "area_mm2", "note"], rows)

print("\nDiscrepancias por revisar:" if problems else "\nSin discrepancias.")
for p in problems:
    print("  -", p)
