import sys
from pathlib import Path
from PIL import Image, ImageDraw

SIZE = 1024


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def draw_icon():
    scale = 4
    s = SIZE * scale
    base = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    gradient = Image.new("RGBA", (s, s))
    top = (46, 108, 230)
    bottom = (16, 40, 96)
    gd = ImageDraw.Draw(gradient)
    for y in range(s):
        gd.line([(0, y), (s, y)], fill=lerp(top, bottom, y / s) + (255,))
    mask = Image.new("L", (s, s), 0)
    margin = int(s * 0.09)
    ImageDraw.Draw(mask).rounded_rectangle([margin, margin, s - margin, s - margin], radius=int(s * 0.2), fill=255)
    base.paste(gradient, (0, 0), mask)
    d = ImageDraw.Draw(base)
    nodes = [(0.27, 0.68), (0.45, 0.42), (0.62, 0.58), (0.76, 0.30)]
    pts = [(int(x * s), int(y * s)) for x, y in nodes]
    d.line(pts, fill=(255, 255, 255, 255), width=int(s * 0.035), joint="curve")
    feasible = [(int(0.27 * s), int(0.76 * s)), (int(0.76 * s), int(0.76 * s)), (int(0.76 * s), int(0.30 * s)), (int(0.62 * s), int(0.58 * s)), (int(0.45 * s), int(0.42 * s)), (int(0.27 * s), int(0.68 * s))]
    overlay = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    ImageDraw.Draw(overlay).polygon(feasible, fill=(255, 255, 255, 60))
    base = Image.alpha_composite(base, overlay)
    d = ImageDraw.Draw(base)
    d.line(pts, fill=(255, 255, 255, 255), width=int(s * 0.035), joint="curve")
    r = int(s * 0.045)
    for i, (x, y) in enumerate(pts):
        fill = (255, 196, 61, 255) if i == len(pts) - 1 else (255, 255, 255, 255)
        d.ellipse([x - r, y - r, x + r, y + r], fill=fill)
    return base.resize((SIZE, SIZE), Image.LANCZOS)


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    icon = draw_icon()
    icon.save(out / "icon.png")
    icon.resize((256, 256), Image.LANCZOS).save(out / "icon_256.png")
    icon.save(out / "icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    iconset = out / "icon.iconset"
    iconset.mkdir(exist_ok=True)
    for size in [16, 32, 128, 256, 512]:
        icon.resize((size, size), Image.LANCZOS).save(iconset / f"icon_{size}x{size}.png")
        icon.resize((size * 2, size * 2), Image.LANCZOS).save(iconset / f"icon_{size}x{size}@2x.png")


if __name__ == "__main__":
    main()
