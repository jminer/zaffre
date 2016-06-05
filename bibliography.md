

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
  