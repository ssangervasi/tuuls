use spectral::assert_that;
use std::path::Path;

use laang::{eval, CliOptions};

#[cfg(not(test))]
mod a {
	static WAT: str = "NOT in test";
}
#[cfg(test)]
mod a {
	pub static WAT: &str = "In test";
}

#[test]
fn test_hello_world() {
	assert_that!(a::WAT).is_equal_to("In test");
	eval(&CliOptions {
		path: Path::new(env!("CARGO_MANIFEST_DIR"))
			.join("examples/hello_world.laang")
			.to_str()
			.unwrap()
			.to_string(),
	});
}
