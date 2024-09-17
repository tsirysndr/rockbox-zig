const { ops } = Deno.core;

const sound = {
  adjustVolume: () => ops.op_adjust_volume(),
  soundSet: () => ops.op_sound_set(),
  soundCurrent: () => ops.op_sound_current(),
  soundDefault: () => ops.op_sound_default(),
  soundMin: () => ops.op_sound_min(),
  soundMax: () => ops.op_sound_max(),
  soundUnit: () => ops.op_sound_unit(),
  soundVal2Phys: () => ops.op_sound_val2phys(),
  getPitch: () => ops.op_get_pitch(),
  setPitch: () => ops.op_set_pitch(),
  beepPlay: () => ops.op_beep_play(),
  pcmbufFade: () => ops.op_pcmbuf_fade(),
  pcmGetLowLatency: () => ops.op_pcm_get_low_latency(),
  systemSoundPlay: () => ops.op_system_sound_play(),
  keyClickClick: () => ops.op_key_click_click(),
};

globalThis.rb = { ...globalThis.rb, sound };
