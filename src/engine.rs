//redis core loop
// waits for commands, parses them and executes/ return the error

use std::cell::RefCell;
use std::io::{self, BufRead, Write};
use std::rc::Rc;

use crate::bridge::ExtensionManager;
use crate::parser::{self, Command};
use crate::storage::Store;

const PROMPT: &str = "> ";

pub fn run<R: BufRead, W: Write>(
    input: R,
    mut output: W,
    store: Rc<RefCell<Store>>,
    extensions: &ExtensionManager,
) -> io::Result<()> {
    let mut lines = input.lines();

    loop {
        write!(output, "{PROMPT}")?;
        output.flush()?;

        let line = match lines.next() {
            Some(line) => line?,
            None => break, // if EOF, exits program
        };

        match parser::parse(&line) {
            Ok(Command::Exit) => break,
            Ok(Command::Add { key, value }) => match extensions.on_add(&key, &value) {
                Ok(final_value) => {
                    store.borrow_mut().add(key, final_value);
                    writeln!(output, "OK")?;
                }
                Err(reason) => writeln!(output, "ERRO: {reason}")?,
            },
            Ok(Command::Get { key }) => {
                let raw_value = store.borrow().get(&key).cloned();
                match raw_value {
                    Some(value) => match extensions.on_get(&key, &value) {
                        Ok(formatted) => writeln!(output, "{formatted}")?,
                        Err(reason) => writeln!(output, "ERRO: {reason}")?,
                    },
                    None => writeln!(output, "ERRO: chave inexistente")?,
                }
            }
            Err(reason) => writeln!(output, "ERRO: {reason}")?,
        }
    }

    Ok(())
}
