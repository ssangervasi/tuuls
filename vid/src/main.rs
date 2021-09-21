use clap::{App, Arg};

use vid::term_screen;
use vid::Options;

fn main() {
    app()
}

fn app() {
    let matches = App::new("Vid Term")
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .required(true)
                .index(1)
                .help("Path to video to play"),
        )
        .arg(
            Arg::with_name("sound")
                .short("s")
                .help("Play sounds using the terminal bell"),
        )
        .arg(
            Arg::with_name("waveform")
                .short("w")
                .help("Display a crude audio waveform next to the video"),
        )
        .get_matches();

    let path = matches.value_of("path").unwrap_or("");
    println!("P: {}", path);

    let sound = matches.occurrences_of("sound") >= 1;
    let waveform = matches.occurrences_of("waveform") >= 1;

    let options = Options {
        path: path.to_string(),
        sound,
        waveform,
    };
    term_screen(&options).unwrap();
}
