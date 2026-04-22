pub fn ease_in_out_expo() -> impl Fn(f32) -> f32 {
    |t: f32| {
        if t == 0.0 {
            return 0.0;
        }
        if t == 1.0 {
            return 1.0;
        }
        if t < 0.5 {
            2.0_f32.powf(20.0 * t - 10.0) / 2.0
        } else {
            (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
        }
    }
}
