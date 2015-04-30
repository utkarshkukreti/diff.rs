/// A fragment of a computed diff.
#[derive(Clone, Debug, PartialEq)]
pub enum Result<T> {
    Left(T),
    Both(T, T),
    Right(T)
}

/// Computes the diff between two slices.
pub fn slice<'a, T: PartialEq>(left: &'a [T],
                               right: &'a [T]) -> Vec<Result<&'a T>> {
    iter(left.iter(), right.iter())
}

fn iter<'a, I, T>(left: I, right: I) -> Vec<Result<T>> where
    I: Clone + Iterator<Item = T> + DoubleEndedIterator, T: PartialEq
{
    let left_count = left.clone().count();
    let right_count = right.clone().count();
    let mut table = vec![vec![0; right_count + 1]; left_count + 1];

    for (i, l) in left.clone().enumerate() {
        for (j, r) in right.clone().enumerate() {
            table[i+1][j+1] = if l == r {
                table[i][j] + 1
            } else {
                std::cmp::max(table[i][j+1], table[i+1][j])
            };
        }
    }

    let mut diff = vec![];
    let (mut i, mut j) = (left_count, right_count);
    let (mut li, mut ri) = (left.clone(), right.clone());
    loop {
        if i > 0 && (j == 0 || table[i][j] == table[i-1][j]) {
            i -= 1;
            diff.push(Result::Left(li.next_back().unwrap()));
        } else if j > 0 && (i == 0 || table[i][j] == table[i][j-1]) {
            j -= 1;
            diff.push(Result::Right(ri.next_back().unwrap()));
        } else if i > 0 && j > 0 {
            i -= 1;
            j -= 1;
            diff.push(Result::Both(li.next_back().unwrap(),
                                   ri.next_back().unwrap()));
        } else {
            break
        }
    }
    diff.reverse();
    diff
}
