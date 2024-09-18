use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_sound,
    ops = [
        op_adjust_volume,
        op_sound_set,
        op_sound_current,
        op_sound_default,
        op_sound_min,
        op_sound_max,
        op_sound_unit,
        op_sound_val2_phys,
        op_get_pitch,
        op_set_pitch,
        op_beep_play,
        op_pcmbuf_fade,
        op_pcm_get_low_latency,
        op_sytsem_sound_play,
        op_keyclick_click,
    ],
    esm = ["src/sound/sound.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sound/lib.rb_sound.d.ts")
}

#[op2(async)]
pub async fn op_adjust_volume() {}

#[op2(async)]
pub async fn op_sound_set() {}

#[op2(async)]
pub async fn op_sound_current() {}

#[op2(async)]
pub async fn op_sound_default() {}

#[op2(async)]
pub async fn op_sound_min() {}

#[op2(async)]
pub async fn op_sound_max() {}

#[op2(async)]
pub async fn op_sound_unit() {}

#[op2(async)]
pub async fn op_sound_val2_phys() {}

#[op2(async)]
pub async fn op_get_pitch() {}

#[op2(async)]
pub async fn op_set_pitch() {}

#[op2(async)]
pub async fn op_beep_play() {}

#[op2(async)]
pub async fn op_pcmbuf_fade() {}

#[op2(async)]
pub async fn op_pcm_get_low_latency() {}

#[op2(async)]
pub async fn op_sytsem_sound_play() {}

#[op2(async)]
pub async fn op_keyclick_click() {}
