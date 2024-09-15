const std = @import("std");

extern fn main_c() c_int;
extern fn parse_args(argc: usize, argv: [*]const [*]const u8) c_int;

pub fn main() !void {
    const args = try std.process.argsAlloc(std.heap.page_allocator);
    defer std.process.argsFree(std.heap.page_allocator, args);

    var argv: [10][*]const u8 = undefined;

    var argc: usize = 0;
    for (args) |arg| {
        argv[argc] = @ptrCast(arg.ptr);
        argc += 1;
    }

    _ = parse_args(argc, &argv);
    _ = main_c();
}
