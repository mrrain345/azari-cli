use std::cell::RefCell;
use std::marker::PhantomData;
use std::path::PathBuf;

thread_local! {
    /// The source file path of a receipt being deserialized on this thread.
    static CURRENT_SOURCE: RefCell<Option<PathBuf>> = RefCell::new(None);
}

/// Returns a clone of the currently active deserialization source path.
pub(crate) fn current_source() -> Option<PathBuf> {
    CURRENT_SOURCE.with(|p| p.borrow().clone())
}

/// RAII guard that installs a source file path into the thread-local
/// deserialization context and removes it on drop.
///
/// Set this before calling into serde.
///
/// # Example
///
/// ```text
/// let _guard = SourcePathGuard::set(path.to_path_buf());
/// let receipt: Receipt = serde_saphyr::from_reader(file)?;
/// ```
pub(crate) struct SourcePathGuard(PhantomData<*const ()>);

impl SourcePathGuard {
    /// Sets `path` as the active deserialization source for this thread and
    /// returns a guard that clears it when dropped.
    pub(crate) fn set_path(path: PathBuf) -> Self {
        let full_path: PathBuf = path.canonicalize().ok().unwrap_or(path);
        CURRENT_SOURCE.with(|p| *p.borrow_mut() = Some(full_path));
        Self(PhantomData)
    }
}

impl Drop for SourcePathGuard {
    fn drop(&mut self) {
        CURRENT_SOURCE.with(|p| *p.borrow_mut() = None);
    }
}
