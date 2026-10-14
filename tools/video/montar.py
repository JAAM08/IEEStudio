# Monta el video: grabación de la app + voz + personaje + subtítulos + resaltados.
import glob, json, math, subprocess, sys, wave
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter

D = sys.argv[1] if len(sys.argv) > 1 else "."
OUT = sys.argv[2] if len(sys.argv) > 2 else D + "/guia.mp4"
FPS = 25
W, H = 1920, 1080
K = W / 1440  # de coordenadas lógicas a pixeles del video

plan = json.load(open(f"{D}/plan.json"))
tiempos = json.load(open(f"{D}/tiempos.json"))
video = sorted(glob.glob(f"{D}/grabacion/*.webm"))[-1]
# Tramos que se conservan de la grabación: se recorta lo que la app tardó en
# responder a los clics, dejando un poco antes de que cambie la pantalla.
keep = []
segs = {}
pos = 0.0
for s in tiempos["segments"]:
    ks = s["start"] if s["ready"] - s["start"] < 2.5 else s["ready"] - 1.2
    ke = s["end"]
    keep.append((ks, ke))
    segs[s["id"]] = {"start": pos, "ready": pos + (s["ready"] - ks), "end": pos + (ke - ks)}
    pos += ke - ks
total = pos
t_ready = 0.0

# ------------------------------------------------------------- voz
SR = 22050
voz = np.zeros(int((total + 1) * SR), dtype=np.float32)
habla = []  # (inicio, fin, segmento)
for s in plan:
    with wave.open(f"{D}/voz/{s['id']}.wav") as w:
        assert w.getframerate() == SR
        a = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(np.float32) / 32768
    t = segs[s["id"]]["ready"] + s["offset"]
    i = int(t * SR)
    voz[i:i + len(a)] += a[: len(voz) - i]
    habla.append((t, t + len(a) / SR, s))
voz = np.clip(voz * 1.05, -1, 1)
with wave.open(f"{D}/voz.wav", "wb") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
    w.writeframes((voz * 32767).astype(np.int16).tobytes())

# energía por cuadro, para la boca
hop = SR // FPS
n = len(voz) // hop
rms = np.sqrt(np.convolve((voz[: n * hop] ** 2).reshape(n, hop).mean(axis=1), np.ones(2) / 2, mode="same"))
ref = np.percentile(rms[rms > 1e-4], 90) if (rms > 1e-4).any() else 1

# ------------------------------------------------------------- recursos
font = lambda f, s: ImageFont.truetype(f"{D}/{f}", s)
F_SUB = font("IBMPlexSans-SemiBold.ttf", 38)
F_TITLE = font("PlexCond-Bold.ttf", 132)
F_KICK = font("IBMPlexSans-SemiBold.ttf", 40)
F_TEXT = font("IBMPlexSans-Regular.ttf", 36)
pers = {(m, e): Image.open(f"{D}/p-{m}-{e}.png").convert("RGBA") for m in ("closed", "half", "open") for e in ("open", "shut")}
PS = 210
pers_small = {k: v.resize((PS, PS), Image.LANCZOS) for k, v in pers.items()}
PB = 420
pers_big = {k: v.resize((PB, PB), Image.LANCZOS) for k, v in pers.items()}
logo = Image.open(f"{D}/icono-512.png").convert("RGBA").resize((150, 150), Image.LANCZOS)
YEL = (245, 197, 24, 255)
BASE = (18, 20, 24)


def wrap(text, f, width):
    words, lines, cur = text.split(), [], ""
    for w in words:
        t = (cur + " " + w).strip()
        if f.getlength(t) <= width: cur = t
        else: lines.append(cur); cur = w
    if cur: lines.append(cur)
    return lines


# subtítulos: trozos de hasta dos líneas, repartidos según su largo
subs = []
for t0, t1, s in habla:
    lines = wrap(s["sub"], F_SUB, 1180)
    chunks = [" ".join(lines[i:i + 2]) for i in range(0, len(lines), 2)]
    lens = [len(c) for c in chunks]; tot = sum(lens); t = t0
    for c, l in zip(chunks, lens):
        d = (t1 - t0) * l / tot
        subs.append((t, t + d, c)); t += d


def card(img, alpha, end=False):
    """Portada (inicio) o cierre a pantalla completa."""
    if alpha <= 0: return
    c = Image.new("RGBA", (W, H), BASE + (255,))
    d = ImageDraw.Draw(c)
    g = Image.new("RGBA", (W, H), (0, 0, 0, 0)); gd = ImageDraw.Draw(g)
    gd.ellipse((W - 1100, H - 520, W + 200, H + 520), fill=(245, 197, 24, 60))
    c.alpha_composite(g.filter(ImageFilter.GaussianBlur(160)))
    c.alpha_composite(logo, (150, 250))
    if not end:
        d.text((150, 440), "IEEStudio", font=F_TITLE, fill=(236, 238, 241))
        d.text((150, 600), "Guía rápida: un proyecto completo", font=F_KICK, fill=YEL)
        d.text((150, 668), "Cálculo de instalaciones eléctricas de baja tensión", font=F_TEXT, fill=(155, 162, 173))
    else:
        d.text((150, 440), "ieestudio.com", font=F_TITLE, fill=(236, 238, 241))
        d.text((150, 600), "Gratis y de código abierto", font=F_KICK, fill=YEL)
        d.text((150, 668), "En el navegador o en Windows", font=F_TEXT, fill=(155, 162, 173))
    if alpha < 1:
        c.putalpha(c.getchannel("A").point(lambda v: int(v * alpha)))
    img.alpha_composite(c)


def spots_layer(img, rects, a):
    if not rects or a <= 0: return
    m = Image.new("L", (W, H), int(150 * a))
    md = ImageDraw.Draw(m)
    for x0, y0, x1, y1 in rects:
        md.rounded_rectangle((x0 * K, y0 * K, x1 * K, y1 * K), radius=14, fill=0)
    dark = Image.new("RGBA", (W, H), (8, 9, 11, 255)); dark.putalpha(m)
    img.alpha_composite(dark)
    d = ImageDraw.Draw(img)
    for x0, y0, x1, y1 in rects:
        d.rounded_rectangle((x0 * K, y0 * K, x1 * K, y1 * K), radius=14, outline=(245, 197, 24, int(255 * a)), width=5)


def ease(x): return max(0.0, min(1.0, x))


# Los tramos se cortan uno por uno a archivos intermedios y luego se unen:
# cortarlos todos en un mismo filtro guarda en memoria la grabación entera.
import os
os.makedirs(f"{D}/tramos", exist_ok=True)
lista = []
for i, (ks, ke) in enumerate(keep):
    out = f"{D}/tramos/t{i:02d}.mp4"
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-ss", f"{ks:.3f}", "-i", video, "-t", f"{ke - ks:.3f}",
                    "-vf", "fps=25,scale=1920:1080,setsar=1", "-an", "-c:v", "libx264", "-preset", "veryfast", "-crf", "14", out], check=True)
    lista.append(f"file '{os.path.abspath(out)}'")
open(f"{D}/tramos/lista.txt", "w").write("\n".join(lista) + "\n")
subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", f"{D}/tramos/lista.txt", "-c", "copy", f"{D}/tramos/base.mp4"], check=True)
total = float(subprocess.check_output(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", f"{D}/tramos/base.mp4"]).decode())
cuts = "[0:v]setsar=1[a];"
ff = subprocess.Popen([
    "ffmpeg", "-v", "error", "-y",
    "-i", f"{D}/tramos/base.mp4",
    "-f", "rawvideo", "-pix_fmt", "rgba", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
    "-i", f"{D}/voz.wav",
    "-filter_complex", cuts + "[a][1:v]overlay=0:0:format=auto,format=yuv420p[v]",
    "-map", "[v]", "-map", "2:a", "-t", f"{total:.3f}",
    "-c:v", "libx264", "-preset", "slow", "-crf", "24", "-tune", "stillimage",
    "-c:a", "aac", "-b:a", "128k", "-ar", "44100", "-movflags", "+faststart", OUT,
], stdin=subprocess.PIPE)

intro = habla[0]
cierre_fin = habla[-1][1]
mouth_level = 0.0
frames = int(total * FPS)
for f in range(frames):
    t = f / FPS
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    # resaltados del segmento en curso
    for s in plan:
        st = segs[s["id"]]["ready"]; en = segs[s["id"]]["end"]
        if st + 0.5 <= t <= en and s["spots"]:
            a = ease((t - st - 0.5) / 0.35) * ease((en - t) / 0.3)
            spots_layer(img, s["spots"], a)
    # boca y ojos
    e = rms[min(f, len(rms) - 1)] / ref
    mouth_level = 0.55 * mouth_level + 0.45 * e
    mouth = "open" if mouth_level > 0.55 else "half" if mouth_level > 0.18 else "closed"
    eyes = "shut" if (t % 3.7) < 0.13 else "open"
    # portada y cierre
    a_intro = 1 - ease((t - 2.3) / 0.6)
    a_end = ease((t - cierre_fin - 0.4) / 0.6)
    card(img, a_intro)
    card(img, a_end, end=True)
    big = max(a_intro, a_end)
    if big > 0.02:
        p = pers_big[(mouth, eyes)].copy()
        if big < 1: p.putalpha(p.getchannel("A").point(lambda v: int(v * big)))
        img.alpha_composite(p, (W - PB - 230, 330))
    if big < 0.98:
        p = pers_small[(mouth, eyes)].copy()
        if big > 0: p.putalpha(p.getchannel("A").point(lambda v: int(v * (1 - big))))
        img.alpha_composite(p, (34, H - PS - 70))
    # subtítulos
    if big < 0.5:
        for s0, s1, text in subs:
            if s0 <= t < s1:
                lines = wrap(text, F_SUB, 1180)
                lh = 50; bw = max(F_SUB.getlength(l) for l in lines) + 56; bh = lh * len(lines) + 30
                x = (W - bw) / 2 + 60; y = H - 70 - bh
                d = ImageDraw.Draw(img)
                d.rounded_rectangle((x, y, x + bw, y + bh), radius=14, fill=(14, 15, 18, 225))
                for k, l in enumerate(lines):
                    d.text((x + 28, y + 14 + k * lh), l, font=F_SUB, fill=(240, 242, 245))
                break
    ff.stdin.write(img.tobytes())
ff.stdin.close()
ff.wait()

# subtítulos en VTT para el reproductor
def ts(x):
    h = int(x // 3600); m = int(x % 3600 // 60); s = x % 60
    return f"{h:02d}:{m:02d}:{s:06.3f}"
with open(OUT.rsplit(".", 1)[0] + ".vtt", "w") as v:
    v.write("WEBVTT\n\n")
    for k, (s0, s1, text) in enumerate(subs, 1):
        v.write(f"{k}\n{ts(s0)} --> {ts(s1)}\n{text}\n\n")
print("listo", OUT, round(total, 1), "s")
