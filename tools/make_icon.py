"""Genera el ícono de VoltForge (assets/icon/) a partir de su geometría.

Un rayo amarillo eléctrico sobre un cuadrado gris carbón con esquinas
redondeadas. Se dibuja a 1024 px y se reduce, para que los bordes salgan
suaves en todos los tamaños.

Uso: python3 tools/make_icon.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent / "assets" / "icon"
S = 1024
CHARCOAL = (29, 32, 36, 255)
EDGE = (49, 54, 61, 255)
YELLOW = (245, 197, 24, 255)

# Rayo en coordenadas de 0 a 1.
BOLT = [
    (0.585, 0.105),
    (0.255, 0.565),
    (0.475, 0.565),
    (0.405, 0.895),
    (0.745, 0.415),
    (0.525, 0.415),
]


def draw(size: int) -> Image.Image:
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    margin = int(S * 0.04)
    radius = int(S * 0.2)
    d.rounded_rectangle(
        (margin, margin, S - margin, S - margin), radius, fill=CHARCOAL, outline=EDGE, width=int(S * 0.012)
    )
    d.polygon([(x * S, y * S) for x, y in BOLT], fill=YELLOW)
    return img.resize((size, size), Image.LANCZOS)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    big = draw(256)
    big.save(ROOT / "voltforge-256.png")
    draw(64).save(ROOT / "voltforge-64.png")
    big.save(ROOT / "voltforge.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])


if __name__ == "__main__":
    main()
