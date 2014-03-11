use std::path::Path;

pub trait IntoPath {
    fn into_path(self) -> Path;
}

impl<'a> IntoPath for &'a str {
    fn into_path(self) -> Path {
        Path::new(self)
    }
}

impl IntoPath for ~str {
    fn into_path(self) -> Path {
        self.as_slice().into_path()
    }
}

impl IntoPath for Path {
    fn into_path(self) -> Path {
        self
    }
}
