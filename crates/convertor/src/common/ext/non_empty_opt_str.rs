pub trait NonEmptyOptStr<T> {
    fn filter_non_empty(&self) -> Option<&str>;
}

impl<T: AsRef<str>> NonEmptyOptStr<Option<T>> for Option<T> {
    fn filter_non_empty(&self) -> Option<&str> {
        self.as_ref()
            .and_then(|s| (!s.as_ref().trim().is_empty()).then_some(s.as_ref().trim()))
    }
}
