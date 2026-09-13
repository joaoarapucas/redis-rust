mod engine;
mod parser;
mod storage;

use std::io;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    engine::run(stdin.lock(), stdout.lock())
}
