#!/usr/bin/env python3
"""Render the site's social card, the image a shared link to contextgeneric.dev shows.

The card is the hero illustration beside the project's name, its tag line, and the site address, at
1200x630. Run it from the website's root after changing the tag line or the hero illustration:

    python3 scripts/social-card/make.py

It needs `google-chrome` to render the page and ImageMagick's `magick` to read the illustration's
background colour and write the JPEG. Commit the resulting `static/img/social-card.jpg`.
"""

import pathlib
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
HERO = ROOT / "static/img/cgp-hero.webp"
OUT = ROOT / "static/img/social-card.jpg"
TAG_LINE = "A language extension for Rust, with pluggable trait implementations at compile-time."


def background_colour():
    """The hero illustration's corner colour, so the card has no visible seam around it."""
    out = subprocess.run(
        ["magick", str(HERO), "-format", "%[hex:p{5,5}]", "info:"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    return "#" + out[:6]


def page(colour):
    return f"""<!doctype html><html><head><meta charset="utf-8"><style>
  body {{ margin: 0; width: 1200px; height: 630px; background: {colour}; display: flex;
         align-items: center; gap: 36px; padding: 0 64px; box-sizing: border-box;
         font-family: system-ui, -apple-system, "Segoe UI", sans-serif; }}
  img {{ width: 500px; flex-shrink: 0; }}
  h1 {{ margin: 0; font-size: 60px; line-height: 1.12; font-weight: 800; color: #5d0705; }}
  p {{ margin: 22px 0 0; font-size: 30px; line-height: 1.4; color: #7a1614; }}
  .url {{ margin-top: 34px; font-size: 26px; font-weight: 700; color: #911f1c; }}
</style></head><body>
  <img src="{HERO.as_uri()}" alt="">
  <div><h1>Context-Generic Programming</h1>
  <p>{TAG_LINE}</p>
  <p class="url">contextgeneric.dev</p></div>
</body></html>"""


def main():
    with tempfile.TemporaryDirectory() as tmp:
        html = pathlib.Path(tmp) / "card.html"
        png = pathlib.Path(tmp) / "card.png"
        html.write_text(page(background_colour()))
        subprocess.run(
            ["google-chrome", "--headless=new", "--disable-gpu", "--hide-scrollbars",
             "--allow-file-access-from-files", "--window-size=1200,630",
             f"--screenshot={png}", html.as_uri()],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        subprocess.run(["magick", str(png), "-quality", "85", str(OUT)], check=True)
    print(f"wrote {OUT.relative_to(ROOT)} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
