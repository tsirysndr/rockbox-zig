const std = @import("std");

pub const MAX_PATH = 260;
pub const MAX_DIR_LEVELS = 10;

const themable_icons = enum(c_int) {
    NOICON = -2,
    Icon_NOICON = -1,
    Icon_Audio,
    Icon_Folder,
    Icon_Playlist,
    Icon_Cursor,
    Icon_Wps,
    Icon_Firmware,
    Icon_Font,
    Icon_Language,
    Icon_Config,
    Icon_Plugin,
    Icon_Bookmark,
    Icon_Preset,
    Icon_Queued,
    Icon_Moving,
    Icon_Keyboard,
    Icon_Reverse_Cursor,
    Icon_Questionmark,
    Icon_Menu_setting,
    Icon_Menu_functioncall,
    Icon_Submenu,
    Icon_Submenu_Entered,
    Icon_Recording,
    Icon_Voice,
    Icon_General_settings_menu,
    Icon_System_menu,
    Icon_Playback_menu,
    Icon_Display_menu,
    Icon_Remote_Display_menu,
    Icon_Radio_screen,
    Icon_file_view_menu,
    Icon_EQ,
    Icon_Rockbox,
    Icon_Last_Themeable,
};

pub const browse_context = extern struct {
    dirfilter: c_int,
    flags: c_uint, // 'unsigned' is translated to 'c_uint'
    title: [*c]const u8, // Nullable C string for the title
    icon: themable_icons, // Enum 'themable_icons', assumed to be defined elsewhere
    root: [*c]const u8, // Const C string for root directory path
    selected: [*c]const u8, // Const C string for selected file name
    buf: [*c]u8, // Buffer for the selected file
    bufsize: usize, // Size of the buffer (translated to 'usize' for portability)
};

const tree_cache = extern struct {
    entries_handle: c_int, // Handle to the entry cache
    name_buffer_handle: c_int, // Handle to the name cache
    max_entries: c_int, // Maximum number of entries in the cache
    name_buffer_size: c_int, // Size of the name buffer (in bytes)
};

pub const tree_context = extern struct {
    currdir: [MAX_PATH]u8, // Fixed-size array for the current directory
    dirlevel: i32, // int in C is c_int in Zig
    selected_item: i32, // Selected file/id3dbitem index
    selected_item_history: [MAX_DIR_LEVELS]c_int, // History of selected items, fixed-size array

    dirfilter: ?*c_int, // Nullable pointer to an int for file use
    filesindir: i32, // Number of files in the directory cache
    dirsindir: i32, // Directory use
    dirlength: i32, // Total number of entries in directory

    currtable: i32,
    currextra: i32,

    sort_dir: i32, // Directory sort order
    out_of_tree: i32, // Shortcut from elsewhere
    cache: tree_cache, // Struct tree_cache, defined elsewhere

    dirfull: bool,
    is_browsing: bool,

    browse: ?*browse_context, // Pointer to browse_context, nullable
};
pub const entry = extern struct {
    name: ?*c_char, // Pointer to the name (nullable)
    attr: c_int, // FAT attributes + file type flags
    time_write: c_uint, // Last write time (unsigned)
    customaction: c_int, // Custom action (for database use)
};

extern fn tree_init() void;
extern fn rockbox_browse(browse: ?*browse_context) c_int;
extern fn rockbox_browse_at(path: [*]const u8) c_int;
extern fn tree_get_context() *tree_context;
extern fn tree_get_entries(t: *tree_context) *entry;
extern fn tree_get_entry_at(t: *tree_context, index: c_int) *entry;

pub fn _rockbox_browse() c_int {
    var browse: browse_context = .{
        .dirfilter = 0,
        .flags = 0,
        .title = "demo",
        .icon = themable_icons.Icon_NOICON,
        .root = "/",
        .selected = null,
        .buf = null,
        .bufsize = 0,
    };
    return rockbox_browse(&browse);
}

pub fn _tree_get_context() tree_context {
    const tc = tree_get_context();
    return tc.*;
}

pub fn _tree_get_entries() *entry {
    const tc = tree_get_context();
    return tree_get_entries(tc);
}

pub fn _tree_get_entry_at(index: c_int) entry {
    const tc = tree_get_context();
    const e = tree_get_entry_at(tc, index);
    return e.*;
}
