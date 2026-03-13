use std::cell::RefCell;
use std::marker::PhantomData;
use std::path::PathBuf;

thread_local! {
    /// Stack of source file paths for receipts being deserialized on this thread.
    /// The most recently pushed path is at the end of the vector.
    static PATH_STACK: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
}

/// Returns a clone of the currently active source path.
pub(crate) fn current_path() -> Option<PathBuf> {
    PATH_STACK.with(|s| s.borrow().last().cloned())
}

/// RAII guard that pushes a source file path onto the thread-local path stack
/// and pops it on drop.
///
/// Set this before calling into serde.
///
/// # Example
///
/// ```text
/// let _guard = SourcePathGuard::push_path(path.to_path_buf());
/// let receipt: Receipt = serde_saphyr::from_reader(file)?;
/// ```
pub(crate) struct SourcePathGuard {
    /// The expected stack depth (length) when this guard is dropped.
    /// Equal to the length of the stack immediately after the path was pushed.
    depth: usize,
    /// A phantom type that makes this struct non-Send and non-Sync.
    ///
    /// **NOTE:** Can be replaced with `impl !Send + !Sync for SourcePathGuard {}` in the future.
    /// Negative trait bounds are not yet available in stable Rust.
    _phantom: PhantomData<*const ()>,
}

impl SourcePathGuard {
    /// Pushes `path` onto the path stack and
    /// returns a guard that pops it when dropped.
    pub(crate) fn push_path(path: PathBuf) -> Self {
        let full_path: PathBuf = path.canonicalize().unwrap_or(path);

        let depth = PATH_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            stack.push(full_path);
            stack.len()
        });

        Self {
            depth,
            _phantom: PhantomData,
        }
    }
}

impl Drop for SourcePathGuard {
    fn drop(&mut self) {
        PATH_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            assert_eq!(
                stack.len(),
                self.depth,
                "SourcePathGuard dropped out of order: expected stack depth {}, found {}.",
                self.depth,
                stack.len(),
            );
            stack.pop();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stack_returns_none() {
        assert_eq!(current_path(), None);
    }

    #[test]
    fn push_sets_current_path() {
        let path = PathBuf::from("/test/path");
        let _guard = SourcePathGuard::push_path(path.clone());
        assert_eq!(current_path(), Some(path));
    }

    #[test]
    fn drop_clears_current_path() {
        let path = PathBuf::from("/test/path");
        let guard = SourcePathGuard::push_path(path);
        drop(guard);
        assert_eq!(current_path(), None);
    }

    #[test]
    fn nested_guards_lifo_order() {
        let path1 = PathBuf::from("/test/path1");
        let path2 = PathBuf::from("/test/path2");

        let guard1 = SourcePathGuard::push_path(path1.clone());
        assert_eq!(current_path(), Some(path1.clone()));

        let guard2 = SourcePathGuard::push_path(path2.clone());
        assert_eq!(current_path(), Some(path2));

        drop(guard2);
        assert_eq!(current_path(), Some(path1));

        drop(guard1);
        assert_eq!(current_path(), None);
    }

    #[test]
    fn out_of_order_drop_panics() {
        use std::mem;
        use std::panic;

        let guard1 = SourcePathGuard::push_path(PathBuf::from("/test/path1"));
        let guard2 = SourcePathGuard::push_path(PathBuf::from("/test/path2"));

        // Forget guard2 without popping, leaving the stack at depth 2.
        // Dropping guard1, which recorded depth=1, must panic.
        mem::forget(guard2);

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| drop(guard1)));

        // Both guards are now gone without having popped their entries; clean up
        // so this thread's stack doesn't affect any tests that follow.
        PATH_STACK.with(|s| s.borrow_mut().clear());

        assert!(
            result.is_err(),
            "expected a panic when a guard is dropped out of order"
        );
    }
}
