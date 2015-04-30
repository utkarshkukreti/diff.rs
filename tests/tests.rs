#![feature(plugin)]
#![plugin(speculate)]

extern crate diff;

speculate! {
    describe "slice" {
        before {
            fn go<T>(left: &[T], right: &[T], len: usize) where
                T: Clone + ::std::fmt::Debug + PartialEq
            {
                let diff = ::diff::slice(&left, &right);
                assert_eq!(diff.len(), len);
                let (mut left_, mut right_) = (vec![], vec![]);
                for d in diff {
                    match d {
                        ::diff::Result::Left(l) => left_.push(l.clone()),
                        ::diff::Result::Both(l, r) => {
                            left_.push(l.clone());
                            right_.push(r.clone());
                        },
                        ::diff::Result::Right(r) => right_.push(r.clone()),
                    }
                }
                assert_eq!(left, &left_[..]);
                assert_eq!(right, &right_[..]);
            }
        }

        test "empty slices" {
            let slice: &[()] = &[];
            go(&slice, &slice, 0);
        }

        test "equal + non-empty slices" {
            let slice = [1, 2, 3];
            go(&slice, &slice, 3);
        }

        test "left empty, right non-empty" {
            let slice = [1, 2, 3];
            go(&slice, &[], 3);
        }

        test "left non-empty, right empty" {
            let slice = [1, 2, 3];
            go(&[], &slice, 3);
        }

        test "misc 1" {
            let left = [1, 2, 3, 4, 1, 3];
            let right = [1, 4, 1, 1];
            go(&left, &right, 7);
        }

        test "misc 2" {
            let left = [1, 2, 1, 2, 3, 2, 2, 3, 1, 3];
            let right = [3, 3, 1, 2, 3, 1, 2, 3, 4, 1];
            go(&left, &right, 14);
        }
    }
}
