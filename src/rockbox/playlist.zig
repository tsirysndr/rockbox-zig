const MAX_PATH = 260;
const PLAYLIST_CONTROL_FILE_SIZE = 256;

const PLAYLIST_PREPEND = -1;
const PLAYLIST_INSERT = -2;
const PLAYLIST_INSERT_LAST = -3;
const PLAYLIST_INSERT_FIRST = -4;
const PLAYLIST_INSERT_SHUFFLED = -5;
const PLAYLIST_REPLACE = -6;
const PLAYLIST_INSERT_LAST_SHUFFLED = -7;
const PLAYLIST_INSERT_LAST_ROTATED = -8;

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

pub const PlaylistInsertContext = extern struct {
    playlist: *PlaylistInfo,
    position: c_int,
    queue: bool,
    progress: bool,
    initialized: bool,
    count: c_int,
    count_langid: c_int,
};

extern fn playlist_get_current() *PlaylistInfo;
extern fn playlist_get_track_info(playlist: *PlaylistInfo, index: c_int, info: *PlaylistTrackInfo) c_int;
extern fn playlist_create(dir: [*]const u8, file: [*]const u8) c_int;
extern fn playlist_insert_context_create(playlist: *PlaylistInfo, context: *PlaylistInsertContext, position: c_int, queue: bool, progress: bool) c_int;
extern fn playlist_insert_context_release(context: *PlaylistInsertContext) void;
extern fn playlist_insert_context_add(context: *PlaylistInsertContext, filename: [*]const u8) c_int;

pub fn _get_track_info_from_current_playlist(index: c_int) PlaylistTrackInfo {
    const playlist = playlist_get_current();
    var info: PlaylistTrackInfo = undefined;
    _ = playlist_get_track_info(playlist, index, &info);
    return info;
}

pub fn build_playlist(files: [*]const [*]const u8, start_index: c_int, size: c_int) c_int {
    const start = start_index;
    const playlist = playlist_get_current();
    var context: PlaylistInsertContext = undefined;

    _ = playlist_insert_context_create(playlist, &context, PLAYLIST_REPLACE, false, false);

    var i: usize = 0;
    while (i < size) {
        const res = playlist_insert_context_add(&context, files[i]);
        if (res < 0) {
            break;
        }
        i += 1;
    }

    playlist_insert_context_release(&context);
    return start;
}
