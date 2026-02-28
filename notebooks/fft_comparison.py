"""
Compare freqshow's FFT output against NumPy's FFT as a reference.

Runs both libraries on the same images and reports per-pixel differences.
Also generates side-by-side spectrum, low-pass, and high-pass visualizations.

Usage:
    python3 notebooks/fft_comparison.py
"""

import subprocess
import numpy as np
from PIL import Image
from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parent.parent
IMG_DIR = ROOT / "data"
OUT_DIR = ROOT / "notebooks" / "output"
OUT_DIR.mkdir(exist_ok=True)

CUTOFF = 0.10
SMOOTHING = 0.02


def load_gray(path: str) -> np.ndarray:
    return np.array(Image.open(path).convert("L"))


def numpy_spectrum(img: np.ndarray) -> np.ndarray:
    """Forward FFT, shift, log magnitude — the NumPy reference."""
    f = np.fft.fft2(img / 255.0)
    fshift = np.fft.fftshift(f)
    magnitude = np.log1p(np.abs(fshift))
    magnitude = (magnitude / magnitude.max() * 255).astype(np.uint8)
    return magnitude


def numpy_low_pass_mask(h: int, w: int, cutoff: float, smoothing: float) -> np.ndarray:
    diagonal = np.sqrt(w * w + h * h)
    r_in = max(0.0, cutoff - smoothing / 2) * diagonal
    r_out = (cutoff + smoothing / 2) * diagonal
    cy, cx = (h - 1) / 2.0, (w - 1) / 2.0
    y, x = np.ogrid[:h, :w]
    d2 = (x - cx) ** 2 + (y - cy) ** 2
    mask = np.where(
        d2 <= r_in**2,
        1.0,
        np.where(d2 >= r_out**2, 0.0, ((r_out**2 - d2) / (r_out**2 - r_in**2)) ** 2),
    )
    return mask


def numpy_filter(img: np.ndarray, mask: np.ndarray) -> np.ndarray:
    f = np.fft.fft2(img / 255.0)
    fshift = np.fft.fftshift(f)
    filtered = fshift * mask
    result = np.fft.ifft2(np.fft.ifftshift(filtered)).real
    return np.clip(result * 255, 0, 255).astype(np.uint8)


def run_rust_example(image_path: str, stem: str):
    """Run the freqshow example and move outputs to the output directory."""
    subprocess.run(
        ["cargo", "run", "--release", "--example", "freq_out", "--", image_path],
        cwd=ROOT,
        capture_output=True,
        check=True,
    )
    result = {}
    for name in ("spectrum", "low_pass", "high_pass"):
        src = ROOT / f"{name}.png"
        dst = OUT_DIR / f"{stem}_{name}.png"
        src.rename(dst)
        result[name] = load_gray(dst)
    return result


def compare(name: str, image_path: Path):
    print(f"\n{'='*60}")
    print(f"  {name} ({image_path.name})")
    print(f"{'='*60}")

    img = load_gray(image_path)
    h, w = img.shape
    print(f"  Size: {w}x{h}")

    # NumPy reference
    np_spectrum = numpy_spectrum(img)
    lp_mask = numpy_low_pass_mask(h, w, CUTOFF, SMOOTHING)
    hp_mask = 1.0 - lp_mask
    np_low_pass = numpy_filter(img, lp_mask)
    np_high_pass = numpy_filter(img, hp_mask)

    # Rust output
    rust = run_rust_example(str(image_path), image_path.stem)

    # Compare
    for label, np_img, rs_img in [
        ("spectrum", np_spectrum, rust["spectrum"]),
        ("low_pass", np_low_pass, rust["low_pass"]),
        ("high_pass", np_high_pass, rust["high_pass"]),
    ]:
        diff = np.abs(np_img.astype(int) - rs_img.astype(int))
        print(f"  {label:10s}  max_diff={diff.max():3d}  mean_diff={diff.mean():.2f}  "
              f"pixels_off_by_more_than_1={np.sum(diff > 1)}")

    # Save side-by-side comparison
    fig, axes = plt.subplots(3, 2, figsize=(10, 15))
    for row, (label, np_img, rs_img) in enumerate([
        ("Spectrum", np_spectrum, rust["spectrum"]),
        ("Low-pass", np_low_pass, rust["low_pass"]),
        ("High-pass", np_high_pass, rust["high_pass"]),
    ]):
        axes[row][0].imshow(np_img, cmap="gray")
        axes[row][0].set_title(f"NumPy {label}")
        axes[row][0].axis("off")
        axes[row][1].imshow(rs_img, cmap="gray")
        axes[row][1].set_title(f"freqshow {label}")
        axes[row][1].axis("off")

    fig.suptitle(name, fontsize=16)
    fig.tight_layout()
    out_path = OUT_DIR / f"{image_path.stem}_comparison.png"
    fig.savefig(out_path, dpi=100)
    plt.close(fig)
    print(f"  Saved comparison → {out_path.relative_to(ROOT)}")


if __name__ == "__main__":
    test_images = [
        ("Mandrill", IMG_DIR / "mandrill.jpg"),
        ("Lena", IMG_DIR / "lena.jpg"),
        ("Vertical stripes", IMG_DIR / "stripes_v.png"),
        ("Horizontal stripes", IMG_DIR / "stripes_h.png"),
        ("Checkerboard", IMG_DIR / "checkerboard.png"),
    ]

    for name, path in test_images:
        if path.exists():
            compare(name, path)
        else:
            print(f"Skipping {name}: {path} not found")

    print(f"\nDone. Comparison images saved to {OUT_DIR.relative_to(ROOT)}/")
