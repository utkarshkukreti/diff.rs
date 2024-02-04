extern crate diff;

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    let myers = args.iter().any(|arg| arg == "--myers");

    let chars = args.iter().any(|arg| arg == "--chars");

    args.retain(|arg| arg != "--myers" && arg != "--chars");

    if args.len() != 2 {
        println!("usage: cargo run --example diff [--myers] <first file> <second file>");
        std::process::exit(1);
    }

    let left = std::fs::read_to_string(&args[0]).unwrap();
    let right = std::fs::read_to_string(&args[1]).unwrap();

    if chars {
        let diff = if myers {
            diff::myers::chars(&left, &right)
        } else {
            diff::chars(&left, &right)
        };

        let mut open = None;

        for d in diff {
            match (d, open) {
                (diff::Result::Left(l), Some("-]")) => print!("{}", l),
                (diff::Result::Left(l), open_) => {
                    print!("{}[-{}", open_.unwrap_or(""), l);
                    open = Some("-]");
                }
                (diff::Result::Right(r), Some("+}")) => print!("{}", r),
                (diff::Result::Right(r), open_) => {
                    print!("{}{{+{}", open_.unwrap_or(""), r);
                    open = Some("+}");
                }
                (diff::Result::Both(l, _), Some(open_)) => {
                    print!("{}{}", open_, l);
                    open = None;
                }
                (diff::Result::Both(l, _), None) => print!("{}", l),
            }
        }
    } else {
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
}
