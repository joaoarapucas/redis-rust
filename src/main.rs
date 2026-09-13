mod bridge;
mod engine;
mod parser;
mod storage;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use storage::Store;

const EXTENSIONS_DIR: &str = "extensions";

fn main() -> io::Result<()> {
    let store = Rc::new(RefCell::new(Store::new()));
    let extensions = bridge::ExtensionManager::load(EXTENSIONS_DIR, Rc::clone(&store));

    let stdin = io::stdin();
    let stdout = io::stdout();
    engine::run(stdin.lock(), stdout.lock(), store, &extensions)
}
