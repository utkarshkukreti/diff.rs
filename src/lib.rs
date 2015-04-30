/// A fragment of a computed diff.
#[derive(Clone, Debug, PartialEq)]
pub enum Result<T> {
    Left(T),
    Both(T, T),
    Right(T)
}
