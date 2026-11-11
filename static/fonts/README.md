# Fonts

The site self-hosts two font families under the SIL Open Font License 1.1, each beside its licence:

- `space-grotesk/`: Space Grotesk, for headings, limited to weights 500 to 700.
- `jetbrains-mono/`: JetBrains Mono, for code, limited to weights 400 to 700, without its ligature
  feature.

Each file is a Latin subset in WOFF2, cut from the variable font in the
[Google Fonts repository](https://github.com/google/fonts/tree/main/ofl) with `fonttools`:

```sh
fonttools varLib.instancer 'SpaceGrotesk[wght].ttf' wght=500:700 -o sg.ttf
pyftsubset sg.ttf --unicodes="$LATIN" --layout-features='kern,liga,mark,mkmk,ss01' \
  --flavor=woff2 --output-file=space-grotesk-latin.woff2
fonttools varLib.instancer 'JetBrainsMono[wght].ttf' wght=400:700 -o jb.ttf
pyftsubset jb.ttf --unicodes="$LATIN,U+2500-257F,U+2022,U+25B6,U+2713,U+2717" \
  --layout-features='kern,mark,mkmk' --flavor=woff2 --output-file=jetbrains-mono-latin.woff2
```

`$LATIN` is
`U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+2074,U+20AC,U+2122,U+2190-2199,U+2212,U+2215,U+2260,U+2264-2265`.
`src/css/custom.css` declares the faces and their metric-matched fallbacks.
