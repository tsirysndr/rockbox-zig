const MAX_PATH = 260;
const PLAYLIST_CONTROL_FILE_SIZE = 256;

pub const PlaylistInfo = extern struct {
    utf8: bool,
    control_created: bool,
    flags: c_uint,
    fd: c_int,
    control_fd: c_int,
    max_playlist_size: c_int,
    indices: ?*c_ulong,
    index: c_int,
    first_index: c_int,
    amount: c_int,
    last_insert_pos: c_int,
    started: bool,
    last_shuffled_start: c_int,
    seed: c_int,
    dcfrefs_handle: c_int,
    dirlen: c_int,
    filename: [MAX_PATH]u8,
    control_filename: [PLAYLIST_CONTROL_FILE_SIZE + 8]u8,
};

pub const PlaylistTrackInfo = extern struct {
    filename: [260]u8,
    attr: c_int,
    index: c_int,
    display_index: c_int,
};

extern fn playlist_get_current() *PlaylistInfo;
extern fn playlist_get_track_info(playlist: *PlaylistInfo, index: c_int, info: *PlaylistTrackInfo) c_int;

pub fn _get_track_info_from_current_playlist(index: c_int) PlaylistTrackInfo {
    const playlist = playlist_get_current();
    var info: PlaylistTrackInfo = undefined;
    _ = playlist_get_track_info(playlist, index, &info);
    return info;
}
