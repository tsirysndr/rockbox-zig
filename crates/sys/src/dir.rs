pub fn open_dir() {
    unsafe {
        crate::opendir();
    }
}

pub fn close_dir() {
    unsafe {
        crate::closedir();
    }
}

pub fn readdir() {
    unsafe { crate::readdir() }
}

pub fn mkdir() {
    unsafe { crate::mkdir() }
}

pub fn rmdir() {
    unsafe { crate::rmdir() }
}

pub fn dir_get_info() {
    unsafe { crate::dir_get_info() }
}
