# Fonts

The site self-hosts two font families under `static/fonts/`, under the SIL Open Font License 1.1, each
beside its licence:

- `inter/`: Inter, for body text and headings, weights 400 to 700, upright and italic, at the
  text optical size. Headings use weight 600.
- `jetbrains-mono/`: JetBrains Mono, for code, weights 400 to 700, upright only and without its
  ligature feature. Code sets at weight 500, and the code themes set comments upright.

Each file is a WOFF2 subset, cut from the variable font in the
[Google Fonts repository](https://github.com/google/fonts/tree/main/ofl) with `fonttools`, run
through `uv run --with fonttools --with brotli`. Both keep Latin; the code font also keeps Greek,
superscripts and subscripts, and box drawing, which the site's code blocks and quoted diagnostics use:

```sh
fonttools varLib.instancer 'Inter[opsz,wght].ttf' wght=400:700 opsz=14 -o inter.ttf
pyftsubset inter.ttf --unicodes="$LATIN" --layout-features='kern,liga,calt,mark,mkmk,ccmp,locl' \
  --flavor=woff2 --output-file=inter-latin.woff2
fonttools varLib.instancer 'Inter-Italic[opsz,wght].ttf' wght=400:700 opsz=14 -o inter-italic.ttf
pyftsubset inter-italic.ttf --unicodes="$LATIN" --layout-features='kern,liga,calt,mark,mkmk,ccmp,locl' \
  --flavor=woff2 --output-file=inter-latin-italic.woff2
fonttools varLib.instancer 'JetBrainsMono[wght].ttf' wght=400:700 -o jb.ttf
pyftsubset jb.ttf --unicodes="$LATIN,U+0370-03FF,U+2070-209F,U+2500-257F,U+2022,U+25B6,U+2713,U+2717" \
  --layout-features='kern,mark,mkmk' --flavor=woff2 --output-file=jetbrains-mono-latin.woff2
```

`$LATIN` is
`U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+2074,U+20AC,U+2122,U+2190-2199,U+2212,U+2215,U+2248,U+2260,U+2264-2265`.
The three files total about 80 KB. A character outside these ranges renders in the fallback face, so
recut the subsets when a page starts using a new script or symbol block.
`src/css/custom.css` declares the faces and their metric-matched fallbacks.
