use serde::{Deserialize, Serialize};

use crate::{cast_ptr, convert_ptr_to_vec, get_string_from_ptr};

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeCache {
    pub entries_handle: i32,     // int entries_handle
    pub name_buffer_handle: i32, // int name_buffer_handle
    pub max_entries: i32,        // int max_entries
    pub name_buffer_size: i32,   // int name_buffer_size (in bytes)
}

impl From<crate::TreeCache> for TreeCache {
    fn from(cache: crate::TreeCache) -> Self {
        Self {
            entries_handle: cache.entries_handle,
            name_buffer_handle: cache.name_buffer_handle,
            max_entries: cache.max_entries,
            name_buffer_size: cache.name_buffer_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeContext {
    pub currdir: String,                 // char currdir[MAX_PATH]
    pub dirlevel: i32,                   // int dirlevel
    pub selected_item: i32,              // int selected_item
    pub selected_item_history: Vec<i32>, // int selected_item_history[MAX_DIR_LEVELS]
    pub filesindir: i32,                 // int filesindir
    pub dirsindir: i32,                  // int dirsindir
    pub dirlength: i32,                  // int dirlength
    pub currtable: i32,                  // int currtable (db use)
    pub currextra: i32,                  // int currextra (db use)
    pub sort_dir: i32,                   // int sort_dir
    pub out_of_tree: i32,                // int out_of_tree
    pub cache: TreeCache,                // struct tree_cache cache
    pub dirfull: bool,                   // bool dirfull
    pub is_browsing: bool,               // bool is_browsing
    pub browse: Option<BrowseContext>,   // struct browse_context* browse
}

impl From<crate::TreeContext> for TreeContext {
    fn from(context: crate::TreeContext) -> Self {
        Self {
            currdir: unsafe {
                std::ffi::CStr::from_ptr(cast_ptr!(context.currdir.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            dirlevel: context.dirlevel,
            selected_item: context.selected_item,
            selected_item_history: context.selected_item_history.to_vec(),
            filesindir: context.filesindir,
            dirsindir: context.dirsindir,
            dirlength: context.dirlength,
            currtable: context.currtable,
            currextra: context.currextra,
            sort_dir: context.sort_dir,
            out_of_tree: context.out_of_tree,
            cache: context.cache.into(),
            dirfull: context.dirfull,
            is_browsing: context.is_browsing,
            browse: None,
            // browse: ptr_to_option!(context.browse).map(|browse| browse.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowseContext {
    pub dirfilter: i32, // int dirfilter
    pub flags: u32,     // unsigned flags
    pub title: String,  // char* title
    // pub icon: ThemableIcons, // enum themable_icons icon
    pub root: String,     // const char* root
    pub selected: String, // const char* selected
    pub buf: Vec<i8>,     // char* buf
    pub bufsize: usize,   // size_t bufsize
}

impl From<crate::BrowseContext> for BrowseContext {
    fn from(context: crate::BrowseContext) -> Self {
        Self {
            dirfilter: context.dirfilter,
            flags: context.flags,
            title: get_string_from_ptr!(context.title),
            root: get_string_from_ptr!(context.root),
            selected: get_string_from_ptr!(context.selected),
            buf: convert_ptr_to_vec!(context.buf, context.bufsize),
            bufsize: context.bufsize,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,      // char* name
    pub attr: i32,         // int attr (FAT attributes + file type flags)
    pub time_write: u32,   // unsigned time_write (Last write time)
    pub customaction: i32, // int customaction (db use)
}

impl From<crate::Entry> for Entry {
    fn from(entry: crate::Entry) -> Self {
        Self {
            name: get_string_from_ptr!(entry.name),
            attr: entry.attr,
            time_write: entry.time_write,
            customaction: entry.customaction,
        }
    }
}
