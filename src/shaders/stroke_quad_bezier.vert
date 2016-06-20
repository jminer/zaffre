#version 140

in vec2 position;
in vec2 pt0;
in vec2 pt1;
in vec2 pt2;
uniform mat4 transform;

out vec2 frag_position;
out vec2 quad_a; // is a constant over every fragment
out vec2 quad_b; // ditto
out vec2 quad_c; // ditto
out float b_d_3a; // ditto
out float p;
out float q;

// Takes coefficients of an equation in the form
//   ax^3 + bx^2 + cx + d = 0
// and returns p and q in the equation
//   t^3 + pt + q = 0
void reduce_to_depressed_cubic(float a, float b, float c, float d, out float p, out float q) {
    // https://en.wikipedia.org/wiki/Cubic_function#Reduction_to_a_depressed_cubic
    // I'm using Wikipedia's formulas because on this page x^3 has no coefficient:
    // http://www.trans4mind.com/personal_development/mathematics/polynomials/cubicAlgebra.htm
    // The only thing it should change compared to the second page is the term added/subtracted at
    // the very end. It needs to be -b/(3*a) instead of -a/3.
    float a_2 = a * a;
    float b_2 = b * b;
    float ac = a * c;
    p = (3.0 * ac - b_2) / (3.0 * a_2);
    q = (2.0 * b_2 * b - 9.0 * ac * b + 27.0 * a_2 * d) / (27.0 * a_2 * a);
}

void main() {
    // To find all points orthogonal to the quadratic curve, we solve the equation
    // (P - B(t)) . B'(t) = 0
    // for P where P is each point, B(t) is the quadratic Bezier function, and B'(t) is its
    // derivative. The equation results in the cubic equation below. See documentation for info.
    vec2 A = pt1 - pt0;             // is a constant over every fragment
    vec2 B = pt0 - 2.0 * pt1 + pt2; // ditto
    vec2 P_prime = pt0 - position;

    frag_position = position;
    quad_a = B;
    quad_b = 2 * A;
    quad_c = pt0;

    // Notice that a and b are constant over every fragment but c and d change because of P_prime
    float a = dot(B, B);
    float b = 3.0 * dot(A, B);
    float c = (2.0 * dot(A, A) + dot(P_prime, B));
    float d = dot(P_prime, A);

    b_d_3a = b / (3.0 * a);
    reduce_to_depressed_cubic(a, b, c, d, p, q);

    gl_Position = transform * vec4(position, 0.0, 1.0);
}
