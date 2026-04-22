pub fn secs_to_slider(position: u64, duration: u64) -> f32 {
    if duration == 0 {
        return 0.0;
    }
    (position as f32 / duration as f32 * 100.0).clamp(0.0, 100.0)
}

pub fn slider_to_secs(value: f32, duration: u64) -> u64 {
    let pct = value.clamp(0.0, 100.0) / 100.0;
    (pct * duration as f32) as u64
}
