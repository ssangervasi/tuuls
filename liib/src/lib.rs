pub const MSG: &str = "Das library";

#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(liib::replace_expr!($tts ())),*])};
}

#[macro_export]
macro_rules! revec {
	( $( $x:expr ),* ) => {
		{
			// let mut temp_vec = Vec::new();
			// $(
			// 	temp_vec.push($x);
			// )*
			// $(
			// 	temp_vec.push($x);
			// )?
			// temp_vec.reverse();
			// let mut size = 0;
			// $(
			// 	 $x;
			// 	size += 1;
			// )*
			let mut size = liib::count_tts!($($x)*);
			println!("size={:?}", size);
			let mut temp_vec = Vec::with_capacity(size);
			$(
				temp_vec.insert(0, $x);
			)*
			temp_vec
		}
	};
}
