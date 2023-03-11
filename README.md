
Zaffre is a 2D rendering library.

## Goals

- A GPU backend using Vulkan similar to pathfinder and piet-gpu
- A CPU backend using tiny-skia for cases when Vulkan isn't available or is too buggy
- A cross-platform text layout and rendering, using the native platform's font and text support
  (DirectWrite on Windows, Core Text on macOS, and FontConfig/Pango/Harfbuzz/FreeType on Linux).

## Status

Currently, the Windows backend is farthest along, but the APIs are designed to work on Linux and
macOS as well. The GPU rendered backend is not usable.

## Testing Fonts

There are three fonts that tests rely on:

- DejaVu Sans - a nice sans serif font with good language support
- Noto Sans Devanagari - for Devanagari support
- Italianno - has metrics and overlaps that most fonts don't
- DejaVu Math TeX Gyre - has non-zero leading and comes with DejaVu Sans

I don't think it's a good idea to use any fonts shipped with Windows or Mac in case there are hacks involved with them. For example, GDI and DirectWrite have hacks for TrueType hinting of fonts of older fonts shipped with Windows (see https://github.com/servo/font-kit/wiki/FAQ).
