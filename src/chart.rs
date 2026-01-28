pub struct Chart {
    height: f64,
    width: f64,
}

impl Chart {
    pub fn new(width: f64, height: f64) -> Self {
        Chart { width, height }
    }
}
