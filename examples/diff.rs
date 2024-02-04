extern crate diff;

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    let myers = args.iter().any(|arg| arg == "--myers");

    args.retain(|arg| arg != "--myers");

    if args.len() != 2 {
        println!("usage: cargo run --example diff [--myers] <first file> <second file>");
        std::process::exit(1);
    }

    let left = std::fs::read_to_string(&args[0]).unwrap();
    let right = std::fs::read_to_string(&args[1]).unwrap();

    let diff = if myers {
        diff::myers::lines(&left, &right)
    } else {
        diff::lines(&left, &right)
    };

    for d in diff {
        match d {
            diff::Result::Left(l) => println!("-{}", l),
            diff::Result::Both(l, _) => println!(" {}", l),
            diff::Result::Right(r) => println!("+{}", r),
        }
    }
}
