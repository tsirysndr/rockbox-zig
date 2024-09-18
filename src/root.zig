const std = @import("std");
const testing = std.testing;

const MAX_PATH = 260; // Define based on the C macro
const PLAYLIST_CONTROL_FILE_SIZE = 256; // Define based on the C macro

//const PlaylistInfo = opaque {};

const PlaylistTrackInfo = extern struct {
    filename: [260]u8,
    attr: c_int,
    index: c_int,
    display_index: c_int,
};

//extern fn playlist_get_current() *PlaylistInfo;
// extern fn playlist_get_track_info(playlist: *PlaylistInfo, index: c_int, info: *PlaylistTrackInfo) c_int;

export fn get_track_info_from_current_playlist(index: c_int) ?*PlaylistTrackInfo {
    std.debug.print("Getting track info from current playlist {}", .{index});
    var info: PlaylistTrackInfo = undefined;
    return &info;
}

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try testing.expect(add(3, 7) == 10);
}
