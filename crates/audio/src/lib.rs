use std::io::Write;
use std::{fs::OpenOptions, path::Path};

const FIFO_PATH: &str = "/tmp/rockbox_audio_fifo";

#[no_mangle]
pub extern "C" fn process_pcm_buffer(data: *mut u8, size: usize) -> i32 {
    if !Path::new(FIFO_PATH).exists() {
        let cstr_path = std::ffi::CString::new(FIFO_PATH).unwrap();
        unsafe {
            if libc::mkfifo(cstr_path.as_ptr(), 0o644) != 0 {
                return -1;
            }
        }

        let mut fifo = match OpenOptions::new().write(true).open(FIFO_PATH) {
            Ok(f) => f,
            Err(_) => return -2,
        };

        let pcm_data = unsafe {
            if data.is_null() {
                return -3;
            }

            std::slice::from_raw_parts(data, size).to_vec()
        };

        if fifo.write_all(&pcm_data).is_err() {
            return -4;
        }
    }

    return 0;
}
