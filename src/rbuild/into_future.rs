use std::path::Path;
use sync::Future;

pub trait IntoFuture<T> {
    fn into_future(self) -> Future<T>;
}

impl<'a> IntoFuture<Path> for &'a str {
    fn into_future(self) -> Future<Path> {
        Path::new(self).into_future()
    }
}

impl IntoFuture<Path> for ~str {
    fn into_future(self) -> Future<Path> {
        Path::new(self).into_future()
    }
}

impl IntoFuture<Path> for Path {
    fn into_future(self) -> Future<Path> {
        Future::from_value(self)
    }
}

impl<T> IntoFuture<T> for Future<T> {
    fn into_future(self) -> Future<T> {
        self
    }
}
