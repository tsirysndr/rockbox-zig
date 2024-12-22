use std::thread;

macro_rules! svec {
    ($($x:expr),*) => (vec![$($x.to_string().into()), *])
}

pub fn repl() {
    let handle = thread::spawn(|| {
        deno::cli(svec!["deno", "repl", "-A"]);
    });
    handle.join().unwrap();
}
