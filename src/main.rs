mod parse;
mod tokenize;

fn main() {
    let mut args = std::env::args().skip(1);

    let file = match args.next() {
        Some(f) => f,
        None => {
            eprintln!("error: no files specified");
            std::process::exit(1);
        }
    };

    let contents = match std::fs::read_to_string(&file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let tokens = match tokenize::tokenize(&contents) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let statements = match parse::parse(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}: {}", file, e);
            std::process::exit(1);
        }
    };

    println!("{:?}", statements);
}
