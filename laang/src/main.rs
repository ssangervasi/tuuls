use std::io::stdout;

use clap::{App, Arg};

use laang::{eval, CliOptions};

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
				.help("Path to .laang"),
		)
		.get_matches();

	let path = matches.value_of("path").unwrap_or("");
	let mut cli_opts = CliOptions {
		path: path.to_string(),
		stdout: stdout(),
	};

	eval(&mut cli_opts)
}
