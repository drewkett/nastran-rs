pub fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let filename = args.next().unwrap();
    println!("{}", filename);

    let data = nastran::op2::parse_file_single(filename).unwrap();
    println!("{:?}", data);
}
