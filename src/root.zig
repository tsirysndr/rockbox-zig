const std = @import("std");
const testing = std.testing;

pub const browse = @import("rockbox/browse.zig");
pub const dir = @import("rockbox/dir.zig");
pub const file = @import("rockbox/file.zig");
pub const lcd = @import("rockbox/lcd.zig");
pub const metadata = @import("rockbox/metadata.zig");
pub const playback = @import("rockbox/playback.zig");
pub const playlist = @import("rockbox/playlist.zig");
pub const settings = @import("rockbox/settings.zig");
pub const sound = @import("rockbox/sound.zig");
pub const system = @import("rockbox/system.zig");
pub const tagcache = @import("rockbox/tagcache.zig");

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

test "basic add functionality" {
    try testing.expect(add(3, 7) == 10);
}
