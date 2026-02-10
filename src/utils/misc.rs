pub fn round_to_precision(value: f64, precision: u32) -> f64 {
    let factor = 10f64.powi(precision as i32);
    (value * factor).round() / factor
}
