#version 140

const float PI = 3.1415926535897932384626433832795;

uniform float half_stroke_width_sq; // (stroke_width / 2)^2

in vec2 frag_position;
in vec2 quad_a;
in vec2 quad_b;
in vec2 quad_c;
in float b_d_3a; // a divided by 3.0
in float p;
in float q;

out vec4 color;

// Finding the cube roots of a complex number:
// http://math.stackexchange.com/a/395435/219881

float cube_root(float n) {
    return sign(n) * pow(abs(n), 1.0 / 3.0);
}

// Finds the point on the quadratic Bézier curve at the specified t value.
vec2 point_at(float t) {
    return quad_a * t * t + quad_b * t + quad_c;
}

// Returns true if the position of this fragment is not within the stroke of the curve at the
// specified t value.
bool is_outside_stroke_at(float t) {
    if(t < 0.0 || t > 1.0)
        return true;

    vec2 pt_vec = point_at(t) - frag_position;
    float dist_sq = dot(pt_vec, pt_vec);
    return dist_sq > half_stroke_width_sq;
}

// TODO: tests with curves where delta is > 0 and < 0 and == 0
void main() {
    // http://www.trans4mind.com/personal_development/mathematics/polynomials/cubicAlgebra.htm
    float p_d_3 = p * (1.0 / 3.0);
    float p_3_d_27 = p_d_3 * p_d_3 * p_d_3;
    float q_d_2 = q * (1.0 / 2.0);

    float delta = q * q * (1.0 / 4.0) + p_3_d_27;
    if(delta <= 0.0) {
        float r = sqrt(-p_3_d_27);
        float phi_d_3 = acos(-q_d_2 / r) * (1.0 / 3.0);
        float two_r_d_3 = 2.0 * sqrt(-p_d_3);

        if(is_outside_stroke_at(two_r_d_3 * cos(phi_d_3) - b_d_3a) &&
           is_outside_stroke_at(two_r_d_3 * cos(phi_d_3 + 2.0 * PI / 3.0) - b_d_3a) &&
           is_outside_stroke_at(two_r_d_3 * cos(phi_d_3 + 4.0 * PI / 3.0) - b_d_3a))
        {
            discard;
        }
    } else {
        float delta_sqrt = sqrt(delta);
        float u = cube_root(-q_d_2 + delta_sqrt);
        float v = cube_root(q_d_2 + delta_sqrt);

        if(is_outside_stroke_at(u - v - b_d_3a))
            discard;
    }
    color = vec4(1.0);
}
