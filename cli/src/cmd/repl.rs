use std::thread;

macro_rules! svec {
    ($($x:expr),*) => (vec![$($x.to_string().into()), *])
}

pub fn repl() {
    let handle = thread::spawn(|| match deno::cli(svec!["deno", "repl", "-A"]) {
        Ok(_) => {}
        Err(_) => {}
    });
    handle.join().unwrap();
}
