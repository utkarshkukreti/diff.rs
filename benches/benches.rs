extern crate criterion;
extern crate diff;

use criterion::Criterion;

criterion::criterion_group!(benches, bench_slice, bench_real_world);
criterion::criterion_main!(benches);

fn bench_slice(c: &mut Criterion) {
    let mut rng = fastrand::Rng::with_seed(0);

    let left = (0..1000).map(|_| rng.u8(..)).collect::<Vec<_>>();

    let swap_10 = swap(&left, 10, &mut rng);
    let swap_50 = swap(&left, 50, &mut rng);
    let swap_100 = swap(&left, 100, &mut rng);
    let swap_500 = swap(&left, 500, &mut rng);
    let swap_1000 = swap(&left, 1000, &mut rng);

    for (name, vec) in [
        ("swap_10", &swap_10),
        ("swap_50", &swap_50),
        ("swap_100", &swap_100),
        ("swap_500", &swap_500),
        ("swap_1000", &swap_1000),
    ] {
        assert_eq!(
            ::diff::slice(&left, vec).len(),
            ::diff::myers::slice(&left, vec).len()
        );

        c.bench_function(&format!("diff::slice {}", name), |b| {
            b.iter(|| ::diff::slice(&left, &vec));
        });

        c.bench_function(&format!("diff::myers::slice {}", name), |b| {
            b.iter(|| ::diff::myers::slice(&left, &vec));
        });
    }

    fn swap<T: Clone>(slice: &[T], swaps: usize, rng: &mut fastrand::Rng) -> Vec<T> {
        let mut vec = slice.to_vec();
        for _ in 0..swaps {
            vec.swap(rng.usize(..slice.len()), rng.usize(..slice.len()));
        }
        vec
    }
}

fn bench_real_world(c: &mut Criterion) {
    let gitignores = std::fs::read_to_string("tests/data/gitignores.txt")
        .unwrap()
        .split("!!!")
        .filter_map(|str| (!str.is_empty()).then(|| str.into()))
        .collect::<Vec<String>>();

    c.bench_function("diff::lines on gitignore files from rust-lang/rust", |b| {
        b.iter(|| {
            for (i, left) in gitignores.iter().enumerate() {
                // diff with previous 3, itself, and next 3
                for right in gitignores[i.saturating_sub(3)..(i + 3).min(gitignores.len())].iter() {
                    ::diff::lines(&left, &right);
                }
            }
        })
    });

    c.bench_function(
        "diff::myers::lines on gitignore files from rust-lang/rust",
        |b| {
            b.iter(|| {
                for (i, left) in gitignores.iter().enumerate() {
                    // diff with previous 3, itself, and next 3
                    for right in
                        gitignores[i.saturating_sub(3)..(i + 3).min(gitignores.len())].iter()
                    {
                        ::diff::myers::lines(&left, &right);
                    }
                }
            })
        },
    );

    c.bench_function("diff::chars on gitignore files from rust-lang/rust", |b| {
        b.iter(|| {
            for (i, left) in gitignores.iter().enumerate() {
                // diff with previous 2, itself, and next 2
                for right in gitignores[i.saturating_sub(2)..(i + 2).min(gitignores.len())].iter() {
                    ::diff::chars(&left, &right);
                }
            }
        })
    });

    c.bench_function(
        "diff::myers::chars on gitignore files from rust-lang/rust",
        |b| {
            b.iter(|| {
                for (i, left) in gitignores.iter().enumerate() {
                    // diff with previous 2, itself, and next 2
                    for right in
                        gitignores[i.saturating_sub(2)..(i + 2).min(gitignores.len())].iter()
                    {
                        ::diff::myers::chars(&left, &right);
                    }
                }
            })
        },
    );
}
