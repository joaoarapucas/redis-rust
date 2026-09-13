use std::env;
use std::fs;

fn main() {
    println!("hello, world!");

    let args: Vec<String> = env::args().collect();

    let cfg = Config::new(&args);

    println!("searching for {}", cfg.query);
    println!("in file {}", cfg.file_path);

    let contents = fs::read_to_string(cfg.file_path).expect("should have been able to read file");

    println!("with text:\n{contents}");
}

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn new(args: &[String]) -> Self {
        let query = args[1].clone();
        let file_path = args[2].clone();

        Config { query, file_path }
    }
}
