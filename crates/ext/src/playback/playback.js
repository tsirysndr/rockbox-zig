const { ops } = Deno.core;

const playback = {
  play: (elapsed, offset) => ops.op_play(elapsed, offset),
  pause: () => ops.op_pause(),
  resume: () => ops.op_resume(),
  next: () => ops.op_next(),
  previous: () => ops.op_previous(),
  fastForwardRewind: () => ops.op_fast_forward_rewind(),
  status: () => ops.op_status(),
  currentTrack: () => ops.op_current_track(),
  flushAndReloadTracks: () => ops.op_flush_and_reload_tracks(),
  getFilePosition: () => ops.op_get_file_position(),
  hardStop: () => ops.op_hard_stop(),
};

globalThis.rb = { ...globalThis.rb, playback };
