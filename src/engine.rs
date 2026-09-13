//redis core loop
// waits for commands, parses them and executes/ return the error

use std::io::{self, BufRead, Write};

use crate::parser::{self, Command};
use crate::storage::Store;

const PROMPT: &str = "> ";

pub fn run<R: BufRead, W: Write>(input: R, mut output: W) -> io::Result<()> {
    let mut store = Store::new(); //hash map
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
            Ok(Command::Add { key, value }) => {
                store.add(key, value);
                writeln!(output, "OK")?;
            }
            Ok(Command::Get { key }) => match store.get(&key) {
                Some(value) => writeln!(output, "{value}")?,
                None => writeln!(output, "ERRO: chave inexistente")?,
            },
            Err(reason) => writeln!(output, "ERRO: {reason}")?,
        }
    }

    Ok(())
}
