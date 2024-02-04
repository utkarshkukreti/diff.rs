#![forbid(unsafe_code)]

use std::marker::PhantomData;

/// A fragment of a computed diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Result<T> {
    /// An element that only exists in the left input.
    Left(T),
    /// Elements that exist in both inputs.
    Both(T, T),
    /// An element that only exists in the right input.
    Right(T),
}

/// Computes the diff between two slices.
pub fn slice<'a, T: PartialEq>(left: &'a [T], right: &'a [T]) -> Vec<Result<&'a T>> {
    do_slice::<_, LeadingTrailing<Naive>>(left, right)
}

/// Computes the diff between the lines of two strings.
pub fn lines<'a>(left: &'a str, right: &'a str) -> Vec<Result<&'a str>> {
    do_lines::<LeadingTrailing<Naive>>(left, right)
}

/// Computes the diff between the chars of two strings.
pub fn chars<'a>(left: &'a str, right: &'a str) -> Vec<Result<char>> {
    do_chars::<LeadingTrailing<Naive>>(left, right)
}

/// Computes diffs using the Myers Diff algorithm [1].
/// This algorithm uses O(M + N) memory and runs in expected O(M + N + D^2)
/// time, where N and M are the lengths of the inputs and D is the number of
/// differences.
/// In practice, this algorithm is faster than the naive algorithm if the inputs
/// have few differences, and always uses much less memory.
/// It is slower than the naive algorithm if the inputs have many differences,
/// which is why this implementation is provided in a separate module and not
/// the default.
/// [1]: https://www.xmailserver.org/diff2.pdf
pub mod myers {
    use super::{LeadingTrailing, Myers, Result};

    /// Computes the diff between two slices.
    pub fn slice<'a, T: PartialEq>(left: &'a [T], right: &'a [T]) -> Vec<Result<&'a T>> {
        super::do_slice::<_, LeadingTrailing<Myers>>(left, right)
    }

    /// Computes the diff between the lines of two strings.
    pub fn lines<'a>(left: &'a str, right: &'a str) -> Vec<Result<&'a str>> {
        super::do_lines::<LeadingTrailing<Myers>>(left, right)
    }

    /// Computes the diff between the chars of two strings.
    pub fn chars<'a>(left: &'a str, right: &'a str) -> Vec<Result<char>> {
        super::do_chars::<LeadingTrailing<Myers>>(left, right)
    }
}

fn do_slice<'a, T: PartialEq, D: Diff>(left: &'a [T], right: &'a [T]) -> Vec<Result<&'a T>> {
    diff::<_, _, D>(left, right, |t| t)
}

fn do_lines<'a, D: Diff>(left: &'a str, right: &'a str) -> Vec<Result<&'a str>> {
    let mut diff = diff::<_, _, D>(
        &left.lines().collect::<Vec<_>>(),
        &right.lines().collect::<Vec<_>>(),
        |str| *str,
    );
    // str::lines() does not yield an empty str at the end if the str ends with
    // '\n'. We handle this special case by inserting one last diff item,
    // depending on whether the left string ends with '\n', or the right one,
    // or both.
    match (
        left.as_bytes().last().cloned(),
        right.as_bytes().last().cloned(),
    ) {
        (Some(b'\n'), Some(b'\n')) => {
            diff.push(Result::Both(&left[left.len()..], &right[right.len()..]))
        }
        (Some(b'\n'), _) => diff.push(Result::Left(&left[left.len()..])),
        (_, Some(b'\n')) => diff.push(Result::Right(&right[right.len()..])),
        _ => {}
    }
    diff
}

fn do_chars<'a, D: Diff>(left: &'a str, right: &'a str) -> Vec<Result<char>> {
    diff::<_, _, D>(
        &left.chars().collect::<Vec<_>>(),
        &right.chars().collect::<Vec<_>>(),
        |char| *char,
    )
}

trait Diff {
    fn diff<'a, T, U, Mapper>(
        left: &'a [T],
        right: &'a [T],
        mapper: &'_ Mapper,
        diff: &'_ mut Vec<Result<U>>,
    ) where
        T: PartialEq,
        Mapper: Fn(&'a T) -> U;
}

fn diff<'a, T, U, D>(left: &'a [T], right: &'a [T], mapper: impl Fn(&'a T) -> U) -> Vec<Result<U>>
where
    T: PartialEq,
    D: Diff,
{
    let mut diff = Vec::with_capacity(left.len().max(right.len()));
    D::diff(left, right, &mapper, &mut diff);
    diff
}

struct LeadingTrailing<Inner: Diff> {
    phantom: PhantomData<Inner>,
}

impl<Inner: Diff> Diff for LeadingTrailing<Inner> {
    fn diff<'a, T, U, Mapper>(
        left: &'a [T],
        right: &'a [T],
        mapper: &'_ Mapper,
        diff: &'_ mut Vec<Result<U>>,
    ) where
        T: PartialEq,
        Mapper: Fn(&'a T) -> U,
    {
        let leading_equals = left.iter().zip(right).take_while(|(l, r)| l == r).count();

        let trailing_equals = left[leading_equals..]
            .iter()
            .rev()
            .zip(right[leading_equals..].iter().rev())
            .take_while(|(l, r)| l == r)
            .count();

        diff.extend(
            left[..leading_equals]
                .iter()
                .zip(right)
                .map(|(l, r)| Result::Both(mapper(l), mapper(r))),
        );

        Inner::diff(
            &left[leading_equals..left.len() - trailing_equals],
            &right[leading_equals..right.len() - trailing_equals],
            &mapper,
            diff,
        );

        diff.extend(
            left[left.len() - trailing_equals..]
                .iter()
                .zip(&right[right.len() - trailing_equals..])
                .map(|(l, r)| Result::Both(mapper(l), mapper(r))),
        );
    }
}

struct Naive {}

impl Diff for Naive {
    fn diff<'a, T, U, Mapper>(
        left: &'a [T],
        right: &'a [T],
        mapper: &'_ Mapper,
        diff: &'_ mut Vec<Result<U>>,
    ) where
        T: PartialEq,
        Mapper: Fn(&'a T) -> U,
    {
        let mut table = Vec2::new(0u32, [left.len() + 1, right.len() + 1]);

        for (i, l) in left.iter().enumerate() {
            for (j, r) in right.iter().enumerate() {
                table.set(
                    [i + 1, j + 1],
                    if l == r {
                        table.get([i, j]) + 1
                    } else {
                        *table.get([i, j + 1]).max(table.get([i + 1, j]))
                    },
                );
            }
        }

        let start = diff.len();

        let mut i = table.len[0] - 1;
        let mut j = table.len[1] - 1;
        loop {
            if j > 0 && (i == 0 || table.get([i, j]) == table.get([i, j - 1])) {
                j -= 1;
                diff.push(Result::Right(mapper(&right[j])));
            } else if i > 0 && (j == 0 || table.get([i, j]) == table.get([i - 1, j])) {
                i -= 1;
                diff.push(Result::Left(mapper(&left[i])));
            } else if i > 0 && j > 0 {
                i -= 1;
                j -= 1;
                diff.push(Result::Both(mapper(&left[i]), mapper(&right[j])));
            } else {
                break;
            }
        }

        diff[start..].reverse();
    }
}

struct Snake {
    d: usize,
    x: usize,
    y: usize,
    u: usize,
    v: usize,
}

impl Snake {
    fn find<T>(left: &[T], right: &[T], forward: &mut [isize], backward: &mut [isize]) -> Self
    where
        T: PartialEq,
    {
        let (n, m) = (left.len() as isize, right.len() as isize);

        debug_assert!(forward.len() >= (n + m + 1) as usize);
        debug_assert!(backward.len() >= (n + m + 1) as usize);

        let delta = n - m;

        let index = |k: isize| (k - 1).rem_euclid(n + m + 1) as usize;

        forward[index(1)] = 0;
        backward[index(1)] = n;

        for d in 0..=((n + m + 1) / 2) {
            for k in (-d..=d).step_by(2) {
                let x = if k == -d || k != d && forward[index(k - 1)] < forward[index(k + 1)] {
                    forward[index(k + 1)]
                } else {
                    forward[index(k - 1)] + 1
                };

                let y = x - k;

                let (mut u, mut v) = (x, y);

                while u < n && v < m && left[u as usize] == right[v as usize] {
                    u += 1;
                    v += 1;
                }

                forward[index(k)] = u;

                if delta.rem_euclid(2) == 1
                    && (delta - (d - 1)..=delta + (d - 1)).contains(&k)
                    && backward[index(delta - k)] <= u
                {
                    debug_assert!(d >= 1 && x >= 0 && y >= 0 && u >= 0 && v >= 0);

                    return Snake {
                        d: (2 * d - 1) as usize,
                        x: x as usize,
                        y: y as usize,
                        u: u as usize,
                        v: v as usize,
                    };
                }
            }

            for k in (-d..=d).step_by(2) {
                let u = if k == -d || k != d && backward[index(k - 1)] > backward[index(k + 1)] {
                    backward[index(k + 1)]
                } else {
                    backward[index(k - 1)] - 1
                };

                let v = u + k - delta;

                let (mut x, mut y) = (u, v);

                while x > 0 && y > 0 && left[(x - 1) as usize] == right[(y - 1) as usize] {
                    x -= 1;
                    y -= 1;
                }

                backward[index(k)] = x;

                if delta.rem_euclid(2) == 0
                    && (-d..=d).contains(&(k - delta))
                    && forward[index(delta - k)] >= x
                {
                    debug_assert!(d >= 0 && x >= 0 && y >= 0 && u >= 0 && v >= 0);

                    return Snake {
                        d: (2 * d) as usize,
                        x: x as usize,
                        y: y as usize,
                        u: u as usize,
                        v: v as usize,
                    };
                }
            }
        }

        unreachable!()
    }
}

struct Myers {}

impl Diff for Myers {
    fn diff<'a, T, U, Mapper>(
        left: &'a [T],
        right: &'a [T],
        mapper: &'_ Mapper,
        diff: &'_ mut Vec<Result<U>>,
    ) where
        T: PartialEq,
        Mapper: Fn(&'a T) -> U,
    {
        let mut buffer = vec![0; 2 * (left.len() + right.len() + 1)];

        let (forward, backward) = buffer.split_at_mut(left.len() + right.len() + 1);

        recur(left, right, mapper, diff, forward, backward);

        fn recur<'a, T, U, Mapper>(
            left: &'a [T],
            right: &'a [T],
            mapper: &Mapper,
            diff: &mut Vec<Result<U>>,
            forward: &mut [isize],
            backward: &mut [isize],
        ) where
            T: PartialEq,
            Mapper: Fn(&'a T) -> U,
        {
            let (n, m) = (left.len(), right.len());
            if n > 0 && m > 0 {
                let snake = Snake::find(left, right, forward, backward);

                debug_assert_eq!(snake.u - snake.x, snake.v - snake.y);

                if snake.d > 1 {
                    recur(
                        &left[..snake.x],
                        &right[..snake.y],
                        mapper,
                        diff,
                        forward,
                        backward,
                    );
                    diff.extend(
                        left[snake.x..snake.u]
                            .iter()
                            .zip(&right[snake.y..snake.v])
                            .map(|(l, r)| Result::Both(mapper(l), mapper(r))),
                    );
                    recur(
                        &left[snake.u..],
                        &right[snake.v..],
                        mapper,
                        diff,
                        forward,
                        backward,
                    );
                } else {
                    diff.extend(
                        left.iter()
                            .zip(right)
                            .map(|(l, r)| Result::Both(mapper(l), mapper(r))),
                    );
                    if m > n {
                        diff.extend(right[n..].iter().map(|r| Result::Right(mapper(r))));
                    } else {
                        diff.extend(left[m..].iter().map(|l| Result::Left(mapper(l))));
                    }
                }
            } else if n > 0 {
                diff.extend(left.iter().map(|l| Result::Left(mapper(l))));
            } else if m > 0 {
                diff.extend(right.iter().map(|r| Result::Right(mapper(r))));
            }
        }
    }
}

struct Vec2<T> {
    len: [usize; 2],
    data: Vec<T>,
}

impl<T> Vec2<T> {
    #[inline]
    fn new(value: T, len: [usize; 2]) -> Self
    where
        T: Clone,
    {
        Vec2 {
            len,
            data: vec![value; len[0] * len[1]],
        }
    }

    #[inline]
    fn get(&self, index: [usize; 2]) -> &T {
        debug_assert!(index[0] < self.len[0]);
        debug_assert!(index[1] < self.len[1]);
        &self.data[index[0] * self.len[1] + index[1]]
    }

    #[inline]
    fn set(&mut self, index: [usize; 2], value: T) {
        debug_assert!(index[0] < self.len[0]);
        debug_assert!(index[1] < self.len[1]);
        self.data[index[0] * self.len[1] + index[1]] = value;
    }
}
