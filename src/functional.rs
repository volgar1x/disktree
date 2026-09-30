pub trait Setter<T> {
    fn set(&mut self, value: T);
}

impl<T> Setter<T> for &mut T {
    fn set(&mut self, value: T) {
        **self = value;
    }
}
