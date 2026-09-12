from PIL import Image, ImageDraw

# pixel-art style launcher: dark rounded panel + 4x4 pixel grid, orange accent
PANEL = (28, 34, 55, 255)
BORDER = (52, 58, 84, 255)
ACCENT = (255, 154, 0, 255)
GRAY = (110, 120, 150, 255)


def icon(px):
    img = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    g = 4
    m = px * 0.20
    cell = (px - 2 * m) / g
    gap = cell * 0.12
    bg = [m - gap *  rententalrental, m - gap * 0, px - m + gap * 0.6, px - m + gap * 0.6]
    bg = [m - gap*0.4] * 4
    d.rounded_rectangle(bg, radius=px*0.16, fill=PANEL)
    for r in range(g):
        for c in range(g):
            x = m + c * cell
            y = m + r * cell
            on = (r + c) % 3 == 0
            col = ACCENT if on else GRAY
            d.rectangle([x + gap/2, y + gap/2, x + cell - gap/2, y + cell - gap/2],
                        fill=None, outline=BORDER, width=max(1, int(px*0.02)))
            inner = [x + gap, y + gap, x + cell - gap, y + cell - gap]
            d.rectangle(inner, fill=col)
    return img


SIZES = [("mdpi", 48), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)]
for name, px in SIZES:
    icon(px).save(f"android/res/mipmap-{name}/ic_launcher.png")
print("launcher icons written:", len(SIZES))
