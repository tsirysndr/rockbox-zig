const std = @import("std");

extern fn main_c() c_int;
extern fn parse_args(argc: usize, argv: [*]const [*]const u8) c_int;

const MAX_PATH = 260; // Define based on the C macro
const PLAYLIST_CONTROL_FILE_SIZE = 256; // Define based on the C macro

const PlaylistInfo = extern struct {
    utf8: bool, // `bool` in C is mapped to `bool` in Zig
    control_created: bool, // Same as above
    flags: c_uint, // `unsigned int` in C is `c_uint` in Zig
    fd: c_int, // `int` in C is `c_int` in Zig
    control_fd: c_int, // Same as above
    max_playlist_size: c_int, // Same as above
    indices: ?*c_ulong, // `unsigned long*` in C is `*c_ulong` in Zig (nullable)
    index: c_int, // `int` in C is `c_int` in Zig
    first_index: c_int, // Same as above
    amount: c_int, // Same as above
    last_insert_pos: c_int, // Same as above
    started: bool, // `bool` in C is `bool` in Zig
    last_shuffled_start: c_int, // `int` in C is `c_int` in Zig
    seed: c_int, // Same as above
    dcfrefs_handle: c_int, // `int` in C is `c_int` in Zig
    dirlen: c_int, // Same as above
    filename: [MAX_PATH]u8, // `char filename[MAX_PATH]` -> `[MAX_PATH]u8` in Zig
    control_filename: [PLAYLIST_CONTROL_FILE_SIZE + 8]u8, // `char control_filename[...]` -> `[...]u8` in Zig
};

const PlaylistTrackInfo = extern struct {
    filename: [260]u8,
    attr: c_int,
    index: c_int,
    display_index: c_int,
};

extern fn playlist_get_current() *PlaylistInfo;
extern fn playlist_get_track_info(playlist: *PlaylistInfo, index: c_int, info: *PlaylistTrackInfo) c_int;

export fn get_track_info_from_current_playlist(index: c_int) PlaylistTrackInfo {
    std.debug.print("Getting track info from current playlist {d}\n", .{index});
    const playlist = playlist_get_current();
    std.debug.print("Got playlist {}\n", .{playlist.*});
    var info: PlaylistTrackInfo = undefined;

    _ = playlist_get_track_info(playlist, index, &info);

    return info;
}

export fn get_current_playlist_amount() c_int {
    const playlist = playlist_get_current();
    return playlist.amount;
}

pub fn main() !void {
    const args = try std.process.argsAlloc(std.heap.page_allocator);
    defer std.process.argsFree(std.heap.page_allocator, args);

    var argv: [10][*]const u8 = undefined;

    var argc: usize = 0;

    if (args.len > 10) {
        std.debug.print("Too many arguments, max 10", .{});
        std.process.exit(1);
    }

    for (args) |arg| {
        argv[argc] = @ptrCast(arg.ptr);
        argc += 1;
    }

    _ = parse_args(argc, &argv);
    _ = main_c();
}
