#[allow(dead_code)]

/// Counter utility methods
pub trait Counter {
    /// Reset the counter.
    fn reset(&mut self);
    /// Increment counter.
    fn increment(&mut self) -> usize;
    /// Decrement counter [unused].
    fn decrement(&mut self) -> usize;
}

impl Counter for usize {
    fn reset(&mut self) {
        *self = 0;
    }

    fn increment(&mut self) -> usize {
        *self = self.checked_add(1).unwrap_or(*self);
        *self
    }

    fn decrement(&mut self) -> usize {
        *self = self.checked_sub(1).unwrap_or(*self);
        *self
    }
}
