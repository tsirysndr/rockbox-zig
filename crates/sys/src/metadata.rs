use std::ffi::{c_uchar, CStr, CString};

use crate::{types::mp3_entry::Mp3Entry, ProgressFunc};

pub fn get_metadata(fd: i32, trackname: &str) -> Mp3Entry {
    let trackname = CString::new(trackname).unwrap();
    let id3 = unsafe { crate::_get_metadata(fd, trackname.as_ptr()) };
    id3.into()
}

pub fn get_codec_string(codectype: i32) -> String {
    let res = unsafe { crate::get_codec_string(codectype) };
    let codec_string = unsafe { CStr::from_ptr(res) };
    codec_string.to_str().unwrap().to_string()
}

pub fn count_mp3_frames(
    fd: i32,
    startpos: i32,
    filesize: i32,
    progressfunc: ProgressFunc,
    buf: *mut c_uchar,
    buflen: usize,
) -> i32 {
    unsafe { crate::count_mp3_frames(fd, startpos, filesize, progressfunc, buf, buflen) }
}

pub fn create_xing_header(
    fd: i32,
    startpos: i64,
    filesize: i64,
    buf: *mut c_uchar,
    num_frames: u64,
    rec_time: u64,
    header_template: u64,
    progressfunc: ProgressFunc,
    generate_toc: bool,
    tempbuf: *mut c_uchar,
    tembuf_len: usize,
) -> i32 {
    let generate_toc = if generate_toc { 1 } else { 0 };
    unsafe {
        crate::create_xing_header(
            fd,
            startpos,
            filesize,
            buf,
            num_frames,
            rec_time,
            header_template,
            progressfunc,
            generate_toc,
            tempbuf,
            tembuf_len,
        )
    }
}
