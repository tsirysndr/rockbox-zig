use std::{ffi::OsString, thread};

pub fn run(args: Vec<OsString>) {
    let handle = thread::spawn(move || match deno::cli(args) {
        Ok(_) => {}
        Err(_) => {}
    });
    handle.join().unwrap();
}
