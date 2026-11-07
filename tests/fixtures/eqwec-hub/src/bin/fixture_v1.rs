//! Non-production v1 fixture executable for Hub end-to-end tests.
//!
//! The executable proves that the activated tree is runnable: it reads the
//! packaged payload from the same tree and prints both its own generation and
//! that payload. A missing payload exits nonzero.

use std::path::Path;

fn main() {
    let executable = std::env::current_exe().expect("resolve fixture executable path");
    let root = executable
        .parent()
        .and_then(Path::parent)
        .expect("fixture executable must live in <tree>/bin");
    let payload = std::fs::read_to_string(root.join("share/fixture.txt"))
        .expect("read share/fixture.txt from the installed tree");
    println!("eqwec-fixture v1");
    print!("{payload}");
}
