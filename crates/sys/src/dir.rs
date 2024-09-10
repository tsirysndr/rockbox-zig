use std::ffi::CString;

use crate::{dirent, Dir, Dirent};

pub fn open_dir(dirname: &str) -> *mut Dir {
    let dirname = CString::new(dirname).unwrap();
    unsafe { crate::opendir(dirname.as_ptr()) }
}

pub fn close_dir(dirp: *mut Dir) -> i32 {
    unsafe { crate::closedir(dirp) }
}

pub fn readdir(dirp: *mut Dir) -> *mut crate::dirent {
    unsafe { crate::readdir(dirp) }
}

pub fn mkdir(path: &str) -> i32 {
    let path = CString::new(path).unwrap();
    unsafe { crate::mkdir(path.as_ptr()) }
}

pub fn rmdir(path: &str) -> i32 {
    let path = CString::new(path).unwrap();
    unsafe { crate::rmdir(path.as_ptr()) }
}

pub fn dir_get_info(dirp: *mut Dir, entry: *mut dirent) -> *mut Dirent {
    unsafe { crate::dir_get_info(dirp, entry) }
}
