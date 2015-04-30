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
    let mut table = vec![vec![0; right.len() + 1]; left.len() + 1];

    for (i, l) in left.iter().enumerate() {
        for (j, r) in right.iter().enumerate() {
            table[i+1][j+1] = if l == r {
                table[i][j] + 1
            } else {
                std::cmp::max(table[i][j+1], table[i+1][j])
            };
        }
    }

    let mut diff = vec![];
    let (mut i, mut j) = (left.len(), right.len());
    loop {
        if i > 0 && (j == 0 || table[i][j] == table[i-1][j]) {
            i -= 1;
            diff.push(Result::Left(&left[i]));
        } else if j > 0 && (i == 0 || table[i][j] == table[i][j-1]) {
            j -= 1;
            diff.push(Result::Right(&right[j]));
        } else if i > 0 && j > 0 {
            i -= 1;
            j -= 1;
            diff.push(Result::Both(&left[i], &right[j]));
        } else {
            break
        }
    }
    diff.reverse();
    diff
}
