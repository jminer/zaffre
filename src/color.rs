
#[derive(Copy, Clone, Debug)]
pub struct Color<N> {
    pub red: N,
    pub green: N,
    pub blue: N,
    pub alpha: N,
}

impl<N: Copy> Color<N> {
    pub fn from_rgba(red: N, green: N, blue: N, alpha: N) -> Self {
        Color { red, green, blue, alpha }
    }

    pub fn as_rgba(&self) -> (N, N, N, N) {
        (self.red, self.green, self.blue, self.alpha)
    }
}

// https://registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html#TRANSFER_SRGB

// SIMD could be used to do sRGB conversion much faster
// https://stackoverflow.com/questions/29856006/sse-intrinsics-convert-32-bit-floats-to-unsigned-8-bit-integers

fn srgb_to_linear(val: u8) -> f32 {
    let val = val as f32 * (1.0 / 255.0);
    if val <= 0.04045 {
        val * (1.0 / 12.92)
    } else {
        ((val + 0.055) * (1.0 / 1.055)).powf(2.4)
    }
}

fn linear_to_srgb(val: f32) -> u8 {
    let val = if val <= 0.0031308 {
        val * 12.92
    } else {
        val.powf(1.0 / 2.4) * 1.055 - 0.055
    };
    (val * 255.0).round() as u8
}
