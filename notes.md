
There is a partial implemention of NV-path:

https://github.com/atilimcetin/path-rendering

However, I see in that code that when you add a cubic Bezier curve, it subdivides the curve into
eight cubic curves. Then it converts each to a quadratic curve and adds them to the path. However,
doing so is only an approximation, as quadratic curves cannot represent all the possible shapes
a cubic curve can:
http://stackoverflow.com/questions/13911963/cubic-bezier-to-quadratic

It should adaptively subdivide into as many quadratic curves as needed to so that the
approximation isn't off by too much.

The current plan for this library:

- Filling will use the approach Blinn and Loop did in their paper. Quadratic and cubic curves will
  be filled with no approximations.
- Stroking will draw quadratic curves exactly. However, cubic curves will be approximated with
  enough quadratic curves that they won't be further than X% of the stroke width off.
