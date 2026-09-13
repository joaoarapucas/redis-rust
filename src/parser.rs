// parses text to commands

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add { key: String, value: String },
    Get { key: String },
    Exit,
}

//given a text, parses it to a command.
// if its invalid, returns the reason only.
pub fn parse(raw_line: &str) -> Result<Command, String> {
    let line = raw_line.trim_start();
    if line.trim().is_empty() {
        return Err("comando vazio".to_string());
    }

    let (head, rest) = split_first_whitespace(line);

    match head {
        "EXIT" => {
            if !rest.trim().is_empty() {
                Err("EXIT não aceita argumentos".to_string())
            } else {
                Ok(Command::Exit)
            }
        }
        "GET" => parse_get(rest),
        "ADD" => parse_add(rest),
        _ => Err(format!("comando desconhecido: {head}")),
    }
}

fn parse_get(rest: &str) -> Result<Command, String> {
    let key = rest.trim();
    if key.is_empty() {
        return Err("uso: GET chave".to_string());
    }
    if key.split_whitespace().count() > 1 {
        return Err("GET aceita apenas uma chave".to_string());
    }
    Ok(Command::Get {
        key: key.to_string(),
    })
}

fn parse_add(rest: &str) -> Result<Command, String> {
    let rest = rest.trim_start();
    let (key, value) = split_first_whitespace(rest);

    if key.is_empty() || value.is_empty() {
        return Err("uso: ADD chave valor".to_string());
    }

    Ok(Command::Add {
        key: key.to_string(),
        value: value.to_string(),
    })
}

// splits key from value using spaces
fn split_first_whitespace(s: &str) -> (&str, &str) {
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}
