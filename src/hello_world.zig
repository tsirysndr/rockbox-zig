const rockbox = @import("./root.zig");

extern const HZ: i32;

export fn hello() i32 {
    rockbox.lcd.splash(HZ * 2, "Hello, World!");
    return 0;
}
