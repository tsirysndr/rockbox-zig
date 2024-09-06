const std = @import("std");

extern fn main_c() c_int;

pub fn main() !void {
    _ = main_c();
}
