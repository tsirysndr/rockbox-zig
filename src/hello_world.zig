const std = @import("std");
const rockbox = @import("./root.zig");

export fn hello(x: i32) i32 {
    _ = rockbox.playback.audioNextTrack();
    _ = rockbox.playback.audioCurrentTrack();
    const status = rockbox.playback.audioStatus();
    std.debug.print("Hello, World! {} {}\n", .{ x, status });
    return 0;
}
