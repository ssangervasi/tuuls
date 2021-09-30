use spectral::assert_that;
use std::path::Path;

use laang::{eval, CliOptions};

#[test]
fn test_vars() {
	let mut opts = CliOptions {
		path: Path::new(env!("CARGO_MANIFEST_DIR"))
			.join("examples/vars.laang")
			.to_str()
			.unwrap()
			.to_string(),
		stdout: Vec::new(),
	};
	eval(&mut opts);
	let out: String = String::from_utf8(opts.stdout).unwrap();
	assert_that!(out).is_equal_to("Hello world family 🗺".to_string());
	// let out = opts.stdout;
	// assert_that!(out).is_equal_to(Vec::from("Asdf"));
}
