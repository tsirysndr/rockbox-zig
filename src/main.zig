const std = @import("std");
const playlist = @import("rockbox/playlist.zig");
const metadata = @import("rockbox/metadata.zig");
const tree = @import("rockbox/tree.zig");

extern fn main_c() c_int;
extern fn parse_args(argc: usize, argv: [*]const [*]const u8) c_int;

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

// metadata functions
export fn rb_get_metadata(fd: c_int, trackname: [*]const u8) metadata.mp3entry {
    return metadata._get_metadata(fd, trackname);
}

// browsing functions
export fn rb_rockbox_browse() c_int {
    return tree._rockbox_browse();
}

export fn rb_tree_get_context() tree.tree_context {
    return tree._tree_get_context();
}

export fn rb_tree_get_entries() *tree.entry {
    return tree._tree_get_entries();
}

export fn rb_tree_get_entry_at(index: c_int) tree.entry {
    return tree._tree_get_entry_at(index);
}

// playlist functions
export fn rb_get_track_info_from_current_playlist(index: c_int) playlist.PlaylistTrackInfo {
    return playlist._get_track_info_from_current_playlist(index);
}

export fn rb_build_playlist(files: [*]const [*]const u8, start_index: c_int, size: c_int) c_int {
    return playlist.build_playlist(files, start_index, size);
}

export fn rb_playlist_insert_track(filename: [*]const u8, position: c_int, queue: bool, sync: bool) c_int {
    return playlist.insert_track(filename, position, queue, sync);
}

export fn rb_playlist_delete_track(index: c_int) c_int {
    return playlist.delete_track(index);
}

export fn rb_playlist_insert_directory(dir: [*]const u8, position: c_int, queue: bool, recurse: bool) c_int {
    return playlist.insert_directory(dir, position, queue, recurse);
}
