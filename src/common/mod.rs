pub mod common {
    use std::cell::RefCell;
    use std::path::Path;

    use crate::Epub;
    pub trait File<T> {
        fn unzip(&self, path: &Path) -> Epub;
    }
}
