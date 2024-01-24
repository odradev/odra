use std::{cell::RefCell, fmt};

/// A display-helper that shows iterators display joined by ",".
#[derive(Debug)]
pub(crate) struct DisplayIter<T>(RefCell<Option<T>>);

impl<T> DisplayIter<T> {
    pub(crate) fn new(item: T) -> Self {
        DisplayIter(RefCell::new(Some(item)))
    }
}

impl<I, T> fmt::Display for DisplayIter<I>
where
    I: IntoIterator<Item = T>,
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(src) = self.0.borrow_mut().take() {
            let mut first = true;
            for item in src.into_iter().take(f.width().unwrap_or(usize::MAX)) {
                if first {
                    first = false;
                    write!(f, "{}", item)?;
                } else {
                    write!(f, ", {}", item)?;
                }
            }

            Ok(())
        } else {
            write!(f, "DisplayIter:GONE")
        }
    }
}

pub mod ds {
    use datasize::DataSize;
    use std::cell::OnceCell;

    pub(crate) fn once_cell<T>(cell: &OnceCell<T>) -> usize
    where
        T: DataSize
    {
        cell.get().map_or(0, |value| value.estimate_heap_size())
    }
}
