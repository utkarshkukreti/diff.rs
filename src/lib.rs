/// A fragment of a computed diff.
pub enum Result<T> {
    Left(T),
    Both(T, T),
    Right(T)
}
