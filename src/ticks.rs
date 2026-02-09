#[derive(Clone, Debug)]
pub struct TickConfig {
    pub min_ticks: usize,
    pub max_ticks: usize,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            min_ticks: 4,
            max_ticks: 10,
        }
    }
}

pub fn compute_ticks(range_min: f32, range_max: f32, config: &TickConfig) -> Vec<f32> {
    if (range_max - range_min).abs() < f32::EPSILON {
        return vec![range_min];
    }

    let (lo, hi) = if range_min < range_max {
        (range_min, range_max)
    } else {
        (range_max, range_min)
    };

    let target = ((config.min_ticks + config.max_ticks) / 2).max(2) as f32;
    let rough_step = (hi - lo) / target;

    let magnitude = 10.0_f32.powf(rough_step.log10().floor());
    let normalized = rough_step / magnitude;

    let nice_factor = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };

    let step = nice_factor * magnitude;

    let start = (lo / step).floor() * step;

    let mut ticks = Vec::new();
    let mut v = start;
    while v <= hi + step * 0.001 {
        ticks.push(v);
        v += step;
    }

    ticks
}
