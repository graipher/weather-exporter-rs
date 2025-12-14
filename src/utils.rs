const B: f32 = 17.368;
const C: f32 = 238.88;

fn gamma(t: f32, rh: f32) -> f32 {
    (rh / 100.0).ln() + B * t / (C + t)
}

pub(crate) fn dew_point_calc(t: f32, rh: f32) -> f32 {
    let g = gamma(t, rh);
    C * g / (B - g)
}
