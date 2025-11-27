fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let name = args.get(1).unwrap();
        println!("Hello {name}");
    } else {
        println!("Hello there");
    }
}
