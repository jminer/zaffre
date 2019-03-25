
#[derive(Copy, Clone, Debug)]
pub struct Color<N> {
    pub red: N,
    pub green: N,
    pub blue: N,
    pub alpha: N,
}

impl<N> Color<N> {
    pub fn new(red: N, green: N, blue: N, alpha: N) -> Self {
        Color { red, green, blue, alpha }
    }
}
