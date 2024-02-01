extern crate diff;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        println!("usage: cargo run --example diff <first file> <second file>");
        std::process::exit(1);
    }

    let left = std::fs::read_to_string(&args[1]).unwrap();
    let right = std::fs::read_to_string(&args[2]).unwrap();

    for diff in diff::lines(&left, &right) {
        match diff {
            diff::Result::Left(l) => println!("-{}", l),
            diff::Result::Both(l, _) => println!(" {}", l),
            diff::Result::Right(r) => println!("+{}", r),
        }
    }
}
