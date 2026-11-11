#!/usr/bin/env python3
"""Check the site theme's colours against WCAG 2 contrast ratios.

Reads the token values for each theme from `src/css/custom.css`, resolves `var()` references, blends
translucent colours over their background, and checks each pair below: text at 4.5:1, and focus rings,
admonition borders, and the two navbar logos at 3:1. It also checks every token colour of the two Prism
themes in `src/prism/themes.ts`, on the code background and on a highlighted line's tint.

Run it from the repository's root after changing a colour:

    python3 scripts/theme/contrast.py

It exits 1 when a pair falls below its minimum.
"""
import re
import sys
from pathlib import Path

# (foreground token, background token, minimum ratio, what the pair is)
PAIRS = [
    ("--cgp-text", "--cgp-bg", 4.5, "body text"),
    ("--cgp-text", "--cgp-surface", 4.5, "body text on a surface"),
    ("--cgp-text", "--cgp-code-bg", 4.5, "plain code text"),
    ("--cgp-text", "--cgp-inline-code-bg", 4.5, "inline code"),
    ("--cgp-text-muted", "--cgp-bg", 4.5, "secondary text"),
    ("--cgp-text-muted", "--cgp-surface", 4.5, "secondary text on a surface"),
    ("--cgp-heading", "--cgp-bg", 4.5, "headings"),
    ("--ifm-color-content-secondary", "--cgp-bg", 4.5, "secondary content, such as the table of contents"),
    ("--ifm-color-primary-contrast-foreground", "--ifm-color-primary-contrast-background", 4.5, "primary alert"),
    ("--cgp-link", "--cgp-bg", 4.5, "links"),
    ("--cgp-link", "--cgp-surface", 4.5, "links on a surface"),
    ("--cgp-link", "--cgp-inline-code-bg", 4.5, "linked inline code"),
    ("--cgp-link", "--cgp-red-50", 4.5, "the active sidebar item"),
    ("--cgp-link-hover", "--cgp-bg", 4.5, "hovered links"),
    ("--cgp-announce-text", "--cgp-announce-bg", 4.5, "the announcement bar"),
    ("--cgp-footer-text", "--cgp-footer-bg", 4.5, "footer text"),
    ("--cgp-footer-link", "--cgp-footer-bg", 4.5, "footer links"),
    ("--cgp-footer-title", "--cgp-footer-bg", 4.5, "footer titles"),
    ("--cgp-focus", "--cgp-bg", 3.0, "the focus ring"),
    ("--cgp-border", "--cgp-bg", 1.2, "borders (decorative, informational only)"),
    ("--ifm-color-info-contrast-foreground", "--ifm-color-info-contrast-background", 4.5, "info admonition"),
    ("--ifm-color-success-contrast-foreground", "--ifm-color-success-contrast-background", 4.5, "tip admonition"),
    ("--ifm-color-secondary-contrast-foreground", "--ifm-color-secondary-contrast-background", 4.5, "note admonition"),
    ("--ifm-color-warning-contrast-foreground", "--ifm-color-warning-contrast-background", 4.5, "warning admonition"),
    ("--ifm-color-danger-contrast-foreground", "--ifm-color-danger-contrast-background", 4.5, "danger admonition"),
    ("--ifm-color-info-dark", "--ifm-color-info-contrast-background", 3.0, "info admonition border and icon"),
    ("--ifm-color-success-dark", "--ifm-color-success-contrast-background", 3.0, "tip admonition border and icon"),
    ("--ifm-color-secondary-dark", "--ifm-color-secondary-contrast-background", 3.0, "note admonition border and icon"),
    ("--ifm-color-warning-dark", "--ifm-color-warning-contrast-background", 3.0, "warning admonition border and icon"),
    ("--ifm-color-danger-dark", "--ifm-color-danger-contrast-background", 3.0, "danger admonition border and icon"),
]
LIGHT_ONLY = [("--cgp-on-red", "--cgp-red", 4.5, "text on a red button")]
REPORT_ONLY = [("--cgp-link", "--cgp-text", "links against body text (links are underlined)")]

BLOCK = re.compile(r"([^{}]+)\{([^{}]*)\}", re.S)
DECL = re.compile(r"(--[\w-]+)\s*:\s*([^;]+);")


def parse(css):
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    themes = {"light": {}, "dark": {}}
    for selector, body in BLOCK.findall(css):
        decls = dict(DECL.findall(body))
        if not decls:
            continue
        if "data-theme='dark'" in selector:
            themes["dark"].update(decls)
        elif ":root" in selector or "data-theme='light'" in selector:
            themes["light"].update(decls)
    # The dark theme inherits every light token it does not redefine, as the cascade does.
    themes["dark"] = {**themes["light"], **themes["dark"]}
    return themes


def resolve(tokens, value, depth=0):
    value = value.strip()
    m = re.fullmatch(r"var\((--[\w-]+)\)", value)
    if m:
        if depth > 10 or m.group(1) not in tokens:
            raise KeyError(m.group(1))
        return resolve(tokens, tokens[m.group(1)], depth + 1)
    return value


def rgba(value):
    value = value.strip()
    if value.startswith("#"):
        h = value[1:]
        if len(h) == 3:
            h = "".join(c * 2 for c in h)
        return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4)) + (1.0,)
    m = re.fullmatch(r"rgba?\(([^)]*)\)", value)
    if m:
        parts = [p.strip() for p in m.group(1).split(",")]
        r, g, b = (int(p) for p in parts[:3])
        a = float(parts[3]) if len(parts) > 3 else 1.0
        return (r, g, b, a)
    raise ValueError(f"cannot read colour {value!r}")


def over(fg, bg):
    a = fg[3]
    return tuple(round(fg[i] * a + bg[i] * (1 - a)) for i in range(3)) + (1.0,)


def lum(c):
    def ch(x):
        x /= 255
        return x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4
    return 0.2126 * ch(c[0]) + 0.7152 * ch(c[1]) + 0.0722 * ch(c[2])


def ratio(a, b):
    la, lb = lum(a), lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def colour(tokens, name, page_bg):
    c = rgba(resolve(tokens, tokens[name]))
    return over(c, page_bg) if c[3] < 1 else c


def prism_themes(path):
    text = Path(path).read_text()
    out = {}
    for theme in ("light", "dark"):
        m = re.search(rf"export const {theme}Theme[^=]*=\s*\{{(.*?)\n\}};", text, re.S)
        if not m:
            raise SystemExit(f"no `export const {theme}Theme` in {path}")
        body = m.group(1)
        plain_bg = re.search(r"plain:\s*\{[^}]*backgroundColor:\s*'([^']+)'", body).group(1)
        plain_fg = re.search(r"plain:\s*\{[^}]*color:\s*'([^']+)'", body).group(1)
        styles = re.findall(r"types:\s*\[([^\]]*)\][^}]*?color:\s*'([^']+)'", body, re.S)
        out[theme] = (plain_bg, plain_fg, [(t.strip(), c) for t, c in styles])
    return out


LOGOS = [("light", "static/img/cgp-logo.svg"), ("dark", "static/img/cgp-logo-dark.svg")]


def logo_fill(path):
    fills = set(re.findall(r"fill:(#[0-9a-fA-F]{6})", Path(path).read_text()))
    if len(fills) != 1:
        raise SystemExit(f"expected one fill colour in {path}, found {sorted(fills)}")
    return fills.pop()


def main(argv):
    site = Path(argv[0]) if argv else Path(__file__).resolve().parents[2]
    themes = parse((site / "src/css/custom.css").read_text())
    failed = 0
    for theme, tokens in themes.items():
        print(f"== {theme}")
        page_bg = rgba(resolve(tokens, tokens["--cgp-bg"]))
        pairs = PAIRS + (LIGHT_ONLY if theme == "light" else [])
        for fg, bg, minimum, what in pairs:
            b = colour(tokens, bg, page_bg)
            f = colour(tokens, fg, b)
            r = ratio(f, b)
            ok = r >= minimum
            failed += not ok
            print(f"  {'ok  ' if ok else 'FAIL'} {r:5.2f} >= {minimum:<3} {what}")
        for fg, bg, what in REPORT_ONLY:
            r = ratio(colour(tokens, fg, page_bg), colour(tokens, bg, page_bg))
            print(f"  info {r:5.2f}        {what}")
        for logo_theme, path in LOGOS:
            if logo_theme == theme:
                r = ratio(rgba(logo_fill(site / path)), page_bg)
                ok = r >= 3.0
                failed += not ok
                print(f"  {'ok  ' if ok else 'FAIL'} {r:5.2f} >= 3.0 the navbar logo, {path}")
    for theme, (bg, fg, styles) in prism_themes(site / "src/prism/themes.ts").items():
        print(f"== prism {theme}")
        b = rgba(bg)
        tokens = themes[theme]
        hl = over(rgba(resolve(tokens, tokens["--cgp-highlight-line"])), b)
        for name, c in [("plain", fg)] + styles:
            r = ratio(rgba(c), b)
            rh = ratio(rgba(c), hl)
            ok = min(r, rh) >= 4.5
            failed += not ok
            print(f"  {'ok  ' if ok else 'FAIL'} {r:5.2f} / {rh:5.2f} highlighted >= 4.5 {name}")
    print("all pairs pass" if not failed else f"{failed} pair(s) below the minimum")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
