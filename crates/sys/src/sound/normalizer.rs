unsafe extern "C" {
    fn pcm_normalizer_enable(enable: bool);
    fn pcm_normalizer_is_enabled() -> bool;
}

pub fn enable(enabled: bool) {
    unsafe { pcm_normalizer_enable(enabled) }
}

pub fn is_enabled() -> bool {
    unsafe { pcm_normalizer_is_enabled() }
}
