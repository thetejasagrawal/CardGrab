#!/usr/bin/env python3
"""Export desktop icons without the edge ringing introduced by Lanczos.

Requires Pillow and macOS iconutil. The transparent master is cardgrabicon.png.
"""

from pathlib import Path
import subprocess
import tempfile

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / "src-tauri" / "icons"
SOURCE = Image.open(ROOT / "cardgrabicon.png").convert("RGBA")


def resize(size: int) -> Image.Image:
    # Pillow filters RGBA in premultiplied-alpha space. Hamming's positive
    # weights avoid detached alpha rows and bright overshoot along the rim.
    return SOURCE.resize((size, size), Image.Resampling.HAMMING)


def validate_edge(image: Image.Image) -> None:
    # A convex icon must not have detached transparent/opaque bands at its
    # top/bottom. Check the horizontal blue section, away from the white hand.
    alpha = image.getchannel("A")
    for x in range(image.width // 3, image.width * 2 // 3):
        column = [alpha.getpixel((x, y)) for y in range(image.height)]
        midpoint = image.height // 2
        assert all(a <= b for a, b in zip(column[:midpoint], column[1:midpoint + 1]))
        assert all(a >= b for a, b in zip(column[midpoint:], column[midpoint + 1:]))


def save_png(path: Path, size: int) -> None:
    image = resize(size)
    validate_edge(image)
    image.save(path, optimize=True)


desktop_sizes = {
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    "StoreLogo.png": 50,
    **{f"Square{size}x{size}Logo.png": size for size in
       (30, 44, 71, 89, 107, 142, 150, 284, 310)},
}
for name, size in desktop_sizes.items():
    save_png(ICONS / name, size)

iconset = ICONS / "icon.iconset"
iconset.mkdir(exist_ok=True)
for size in (16, 32, 128, 256, 512):
    for scale in (1, 2):
        suffix = "@2x" if scale == 2 else ""
        save_png(iconset / f"icon_{size}x{size}{suffix}.png", size * scale)

subprocess.run(["iconutil", "--convert", "icns", str(iconset), "--output",
                str(ICONS / "icon.icns")], check=True)

# Supply every ICO frame explicitly so Pillow never falls back to Lanczos.
ico_sizes = (16, 24, 32, 48, 64, 128, 256)
frames = [resize(size) for size in ico_sizes]
frames[-1].save(ICONS / "icon.ico", sizes=[(n, n) for n in ico_sizes],
                append_images=frames[:-1])

# Verify the actual packaged ICNS, including every Retina representation.
with tempfile.TemporaryDirectory() as directory:
    unpacked = Path(directory) / "check.iconset"
    subprocess.run(["iconutil", "--convert", "iconset", str(ICONS / "icon.icns"),
                    "--output", str(unpacked)], check=True)
    for path in unpacked.glob("*.png"):
        decoded = Image.open(path).convert("RGBA")
        validate_edge(decoded)
        expected = Image.open(iconset / path.name).convert("RGBA")
        assert decoded.getchannel("A").tobytes() == expected.getchannel("A").tobytes(), path.name
        # iconutil re-encodes color data in the legacy non-Retina 16/32 slots.
        # All PNG-based (including Retina) slots must preserve every channel.
        if path.name not in {"icon_16x16.png", "icon_32x32.png"}:
            assert decoded.tobytes() == expected.tobytes(), path.name

print("Exported desktop icons and verified all packaged ICNS edges.")
