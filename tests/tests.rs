extern crate diff;
extern crate quickcheck;

use diff::Result::*;

pub fn undiff<T: Clone>(diff: &[::diff::Result<&T>]) -> (Vec<T>, Vec<T>) {
    let (mut left, mut right) = (vec![], vec![]);
    for d in diff {
        match *d {
            Left(l) => left.push(l.clone()),
            Both(l, r) => {
                left.push(l.clone());
                right.push(r.clone());
            }
            Right(r) => right.push(r.clone()),
        }
    }
    (left, right)
}

pub fn undiff_lines<'a>(diff: &[::diff::Result<&'a str>]) -> (String, String) {
    let (mut left, mut right) = (vec![], vec![]);
    for d in diff {
        match *d {
            Left(l) => left.push(l),
            Both(l, r) => {
                left.push(l);
                right.push(r);
            }
            Right(r) => right.push(r),
        }
    }
    (left.join("\n"), right.join("\n"))
}

pub fn undiff_chars(diff: &[::diff::Result<char>]) -> (String, String) {
    let (mut left, mut right) = (String::new(), String::new());
    for d in diff {
        match *d {
            Left(l) => left.push(l),
            Both(l, r) => {
                left.push(l);
                right.push(r);
            }
            Right(r) => right.push(r),
        }
    }
    (left, right)
}

#[test]
fn test_slice() {
    fn go<T>(left: &[T], right: &[T], len: usize)
    where
        T: Clone + ::std::fmt::Debug + PartialEq,
    {
        let diff = ::diff::slice(left, right);
        assert_eq!(diff.len(), len);
        let (left_, right_) = undiff(&diff);
        assert_eq!(left, &left_[..]);
        assert_eq!(right, &right_[..]);
    }

    let slice: &[()] = &[];
    go(slice, slice, 0);

    let slice = [1, 2, 3];
    go(&slice, &slice, 3);

    let slice = [1, 2, 3];
    go(&slice, &[], 3);

    let slice = [1, 2, 3];
    go(&[], &slice, 3);

    let left = [1, 2, 3, 4, 1, 3];
    let right = [1, 4, 1, 1];
    go(&left, &right, 7);

    let left = [1, 2, 1, 2, 3, 2, 2, 3, 1, 3];
    let right = [3, 3, 1, 2, 3, 1, 2, 3, 4, 1];
    go(&left, &right, 14);

    let left = [1, 3, 4];
    let right = [2, 3, 4];
    go(&left, &right, 4);
}

#[test]
fn test_slice_quickcheck() {
    fn prop(left: Vec<i32>, right: Vec<i32>) -> bool {
        let diff = ::diff::slice(&left, &right);
        let (left_, right_) = undiff(&diff);
        left == left_[..] && right == right_[..]
    }

    ::quickcheck::quickcheck(prop as fn(Vec<i32>, Vec<i32>) -> bool);
}

#[test]
fn test_lines() {
    fn go(left: &str, right: &str, len: usize) {
        let diff = ::diff::lines(left, right);
        assert_eq!(diff.len(), len);
        let (left_, right_) = undiff_lines(&diff);
        assert_eq!(left, left_);
        assert_eq!(right, right_);
    }

    go("", "", 0);

    go("foo", "", 1);
    go("", "foo", 1);

    go("foo\nbar", "foo\nbar", 2);

    go("foo\nbar\nbaz", "foo\nbaz\nquux", 4);

    // #10
    go("a\nb\nc", "a\nb\nc\n", 4);
    go("a\nb\nc\n", "a\nb\nc", 4);
    let left = "a\nb\n\nc\n\n\n";
    let right = "a\n\n\nc\n\n";
    go(left, right, 8);
    go(right, left, 8);
}

#[test]
fn test_chars() {
    fn go(left: &str, right: &str, len: usize) {
        let diff = ::diff::chars(left, right);
        assert_eq!(diff.len(), len);
        let (left_, right_) = undiff_chars(&diff);
        assert_eq!(left, left_);
        assert_eq!(right, right_);
    }

    go("", "", 0);

    go("foo", "", 3);
    go("", "foo", 3);

    go("foo bar", "foo bar", 7);

    go("foo bar baz", "foo baz quux", 16);
}

#[test]
fn test_issue_4() {
    assert_eq!(::diff::slice(&[1], &[2]), vec![Left(&1), Right(&2)]);
    assert_eq!(::diff::lines("a", "b"), vec![Left("a"), Right("b")]);
}

#[test]
fn test_issue_6() {
    // This produced an overflow in the lines computation because it
    // was not accounting for the fact that the "right" length was
    // less than the "left" length.
    let expected = r#"
BacktraceNode {
    parents: [
        BacktraceNode {
            parents: []
        },
        BacktraceNode {
            parents: [
                BacktraceNode {
                    parents: []
                }
            ]
        }
    ]
}"#;
    let actual = r#"
BacktraceNode {
    parents: [
        BacktraceNode {
            parents: []
        },
        BacktraceNode {
            parents: [
                BacktraceNode {
                    parents: []
                },
                BacktraceNode {
                    parents: []
                }
            ]
        }
    ]
}"#;
    ::diff::lines(actual, expected);
}

#[test]
fn gitignores() {
    let all = std::fs::read_to_string("tests/data/gitignores.txt")
        .unwrap()
        .split("!!!")
        .map(|str| str.trim())
        .filter(|str| !str.is_empty())
        .map(|str| str.replace("\r", ""))
        .collect::<Vec<String>>();

    go(
        &all,
        ::diff::lines,
        undiff_lines,
        |d| match d {
            Left(l) => format!("-{}\n", l),
            Right(r) => format!("+{}\n", r),
            Both(l, _) => format!(" {}\n", l),
        },
        "tests/data/gitignores.lines.diff",
    );

    go(
        &all,
        ::diff::chars,
        undiff_chars,
        |d| match d {
            Left(l) => format!("[-{}-]", l),
            Right(r) => format!("{{+{}+}}", r),
            Both(l, _) => format!("{}", l),
        },
        "tests/data/gitignores.chars.diff",
    );

    fn go<'a, T, Diff, Undiff, ToString>(
        all: &'a [String],
        diff: Diff,
        undiff: Undiff,
        to_string: ToString,
        path: &str,
    ) where
        Diff: Fn(&'a str, &'a str) -> Vec<::diff::Result<T>>,
        Undiff: Fn(&[::diff::Result<T>]) -> (String, String),
        ToString: Fn(&::diff::Result<T>) -> String,
        T: 'a,
    {
        let mut actual = String::new();
        for i in 0..all.len() {
            let from = i.saturating_sub(2);
            let to = (i + 2).min(all.len());
            for j in from..to {
                actual.push_str(&format!("i = {}, j = {}\n", i, j));
                let diff = diff(&all[i], &all[j]);
                for d in &diff {
                    actual.push_str(&to_string(d));
                }
                actual.push_str("\n");
                let undiff = undiff(&diff);
                assert_eq!(&undiff.0, &all[i]);
                assert_eq!(&undiff.1, &all[j]);
            }
        }

        if let Ok(expected) = std::fs::read_to_string(path) {
            assert_eq!(expected.replace("\r", ""), actual);
        } else {
            std::fs::write(path, actual).unwrap();
        }
    }
}
