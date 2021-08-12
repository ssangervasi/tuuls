use liib;

fn main() {
    println!("{}", liib::MSG);

    // let v: Vec<u8> = liib::revec![1, 2, 3];
    let v: Vec<u8> = liib::revec![n(1), n(2), 3];
    print!("{:?}", v)
}

fn n(ni: u8) -> u8 {
    println!("n({})", ni);
    ni
}
