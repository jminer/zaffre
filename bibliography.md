

https://www.latex4technics.com/ is a nice LaTeX editor.

- For the overall approach:

  GPU-accelerated Path Rendering
  Kilgard and Bolz 2012
  http://developer.download.nvidia.com/devzone/devcenter/gamegraphics/files/opengl/gpupathrender.pdf

- To determine how many quadratic curves are necessary to approximate a cubic curve, the
  approach from this paper is used to find the error:

  Explicit Error Bound for Quadratic Spline Approximation of Cubic Spline
  Kim and Ahn 2009
  http://ocean.kisti.re.kr/downfile/volume/ksiam/E1TAAE/2009/v13n4/E1TAAE_2009_v13n4_257.pdf

- To fill quadratic and cubic curves, the approach in this paper is used:

  Resolution Independent Curve Rendering using Programmable Graphics Hardware
  Loop and Blinn 2005
  http://research.microsoft.com/pubs/78197/p1000-loop.pdf

- To find the distance to a quadratic curve in the fragment shader, this page was helpful:

  http://blog.gludion.com/2009/08/distance-to-quadratic-bezier-curve.html

- To solve a depressed cubic equation using Cardano's formula, Wikipedia has some information, but
  overall is poor. This page is very useful for solving depressed a cubic equation:

  http://www.trans4mind.com/personal_development/mathematics/polynomials/cubicAlgebra.htm

  This page shows solving a non-depressed cubic:

  To Solve a Cubic Equation
  http://www.codeproject.com/Articles/798474/To-Solve-a-Cubic-Equation

  And in this section, there's some code:

  https://pomax.github.io/bezierinfo/#extremities

- To find the axis of a quadratic curve (which is the point of greatest curvature):

  An Inexpensive Bounding Representation for Offsets of Quadratic Curves
  Ruf 2011
  https://www.researchgate.net/profile/Erik_Ruf/publication/221249023_An_Inexpensive_Bounding_Representation_for_Offsets_of_Quadratic_Curves/links/551c1fcd0cf2fe6cbf7684c0.pdf
