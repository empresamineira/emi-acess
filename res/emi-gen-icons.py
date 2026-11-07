"""Generate EMI Acess icons for Linux, macOS, iOS and Android from the logo.

Usage: python -I res/emi-gen-icons.py <logo.png> .   (needs Pillow; run from the repo root)
The Windows .ico files are not produced here; see EMI-CUSTOM.md.
"""
import base64
import io
import os
import sys

from PIL import Image, ImageDraw

LOGO, ROOT = sys.argv[1], sys.argv[2]
src = Image.open(LOGO).convert("RGBA")
src = src.crop(src.getbbox())
side = max(src.size)
square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
square.paste(src, ((side - src.width) // 2, (side - src.height) // 2), src)


def p(*parts):
    return os.path.join(ROOT, *parts)


def logo(size, scale=1.0):
    """Logo centered on a transparent canvas, occupying `scale` of the side."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inner = max(1, round(size * scale))
    resized = square.resize((inner, inner), Image.LANCZOS)
    off = (size - inner) // 2
    canvas.paste(resized, (off, off), resized)
    return canvas


def on_shape(size, shape, scale, radius=0.22):
    """Logo on a white circle or rounded square, transparent outside (supersampled)."""
    big = size * 4
    mask = Image.new("L", (big, big), 0)
    d = ImageDraw.Draw(mask)
    if shape == "circle":
        d.ellipse((0, 0, big - 1, big - 1), fill=255)
    else:
        d.rounded_rectangle((0, 0, big - 1, big - 1), radius=round(big * radius), fill=255)
    mask = mask.resize((size, size), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (255, 255, 255, 0))
    white = Image.new("RGBA", (size, size), (255, 255, 255, 255))
    canvas.paste(white, (0, 0), mask)
    mark = logo(size, scale)
    return Image.alpha_composite(canvas, mark)


def white_silhouette(size, scale, mode="LA"):
    alpha = logo(size, scale).getchannel("A")
    if mode == "LA":
        return Image.merge("LA", (Image.new("L", (size, size), 255), alpha))
    return Image.merge("RGBA", (Image.new("L", (size, size), 255),) * 3 + (alpha,))


def save(img, *parts):
    path = p(*parts)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.save(path, optimize=True)
    print("wrote", os.path.relpath(path, ROOT), img.size, img.mode)


# Linux (hicolor theme icons, deb and AppImage).
for n, name in [(32, "32x32.png"), (64, "64x64.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
    save(logo(n, 0.94), "res", name)
buf = io.BytesIO()
logo(512, 0.94).save(buf, "PNG", optimize=True)
svg = (
    '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" '
    'width="512" height="512" viewBox="0 0 512 512">'
    '<image width="512" height="512" xlink:href="data:image/png;base64,'
    + base64.b64encode(buf.getvalue()).decode()
    + '"/></svg>\n'
)
with open(p("res", "scalable.svg"), "w", newline="\n") as f:
    f.write(svg)
print("wrote res/scalable.svg", len(svg), "bytes")

# macOS: app icon (rounded white tile, macOS grid ~ 824/1024) and the tray fallback.
mac = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
tile = on_shape(824, "rounded", 0.78)
mac.paste(tile, (100, 100), tile)
mac.save(p("flutter", "macos", "Runner", "AppIcon.icns"),
         sizes=[(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)])
print("wrote flutter/macos/Runner/AppIcon.icns")
save(white_silhouette(60, 0.9, "RGBA"), "res", "mac-tray-dark-x2.png")

# iOS: opaque (no alpha) icons on white.
ios_dir = ("flutter", "ios", "Runner", "Assets.xcassets", "AppIcon.appiconset")
for fn in os.listdir(p(*ios_dir)):
    if not fn.endswith(".png"):
        continue
    w = Image.open(p(*ios_dir, fn)).size[0]
    bg = Image.new("RGBA", (w, w), (255, 255, 255, 255))
    save(Image.alpha_composite(bg, logo(w, 0.8)).convert("RGB"), *ios_dir, fn)

# Android launcher icons, adaptive foreground (108dp, 66dp safe zone) and notification icon.
res = ("flutter", "android", "app", "src", "main", "res")
for density, k in [("mdpi", 1), ("hdpi", 1.5), ("xhdpi", 2), ("xxhdpi", 3), ("xxxhdpi", 4)]:
    d = res + ("mipmap-" + density,)
    save(on_shape(round(48 * k), "rounded", 0.78, 0.18), *d, "ic_launcher.png")
    save(on_shape(round(48 * k), "circle", 0.66), *d, "ic_launcher_round.png")
    save(logo(round(108 * k), 0.58), *d, "ic_launcher_foreground.png")
    save(white_silhouette(round(24 * k), 0.92), *d, "ic_stat_logo.png")
# Floating window: replaces the vector logo; xxhdpi so the intrinsic size stays 320dp.
save(on_shape(960, "circle", 0.66), *res, "drawable-xxhdpi", "floating_window.png")
