use clap::{App, Arg};

use vid::term_screen;

fn main() {
    app()
}

fn app() {
    let matches = App::new("Vid Term")
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .required(true)
                .help("Path to video to play"),
        )
        .get_matches();
    let path = matches.value_of("path").unwrap_or("");
    println!("P: {}", path);
    term_screen(path).unwrap();
}
