#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
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
			let size = count_tts!($($x)*);
			println!("size={:?}", size);
			let mut temp_vec = Vec::with_capacity(size);
			// This is probably _worse_ than calling .reverse :D
			$(
				temp_vec.insert(0, $x);
			)*
			temp_vec
		}
	};
}

pub fn macro_test() {
    // let v: Vec<u8> = revec![1, 2, 3];
    let v: Vec<u8> = revec![n(1), n(2), n(3)];
    print!("{:?}", v)
}

pub fn n(ni: u8) -> u8 {
    println!("n({})", ni);
    ni
}

pub const MSG: &str = "Das library";
