const { ops } = Deno.core;

const playlist = {
  getCurrent: () => ops.op_playlist_get_current(),
  getResumeInfo: () => ops.op_playlist_get_resume_info(),
  getTrackInfo: () => ops.op_playlist_get_track_info(),
  getFirstIndex: () => ops.op_playlist_get_first_index(),
  getDisplayIndex: () => ops.op_playlist_get_display_index(),
  amount: () => ops.op_playlist_amount(),
  playlistResume: () => ops.op_playlist_resume(),
  resumeTrack: () => ops.op_playlist_resume_track(),
  setModified: () => ops.op_playlist_set_modified(),
  start: () => ops.op_playlist_start(),
  sync: () => ops.op_playlist_sync(),
  removeAllTracks: () => ops.op_playlist_remove_all_tracks(),
  createPlaylist: () => ops.op_create_playlist(),
  insertTrack: () => ops.op_playlist_insert_track(),
  insertDirectory: () => ops.op_playlist_insert_directory(),
  insertPlaylist: () => ops.op_insert_playlist(),
  shufflePlaylist: () => ops.op_shuffle_playlist(),
  warnOnPlaylistErase: () => ops.op_warn_on_playlist_erase(),
};

globalThis.rb = { ...globalThis.rb, playlist };
