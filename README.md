
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

There are two fonts that tests rely on: DejaVu Sans and Italianno. DejaVu Sans is a standard font
with good language support that isn't shipped with Windows or Mac. (Some of the fonts that ship with
Windows have hacks in them that could cause the metrics to vary if they were used on Linux; they
aren't suitable for use with tests.) Italianno has metrics and overlaps that most fonts don't. And
both are freely available.
