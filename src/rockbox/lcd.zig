const plugin_api = extern struct {
    splash: *const fn (ticks: c_int, str: [*:0]const u8) callconv(.C) void,
};

extern const rb: *plugin_api;

pub fn splash(ticks: i32, str: [*:0]const u8) void {
    rb.splash(ticks, str);
}
