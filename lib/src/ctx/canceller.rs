use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Canceller {
	/// A reference to the canceled value of a context.
	cancelled: Arc<AtomicBool>,
}

impl Canceller {
	/// Create a new Canceller
	pub fn new(cancelled: Arc<AtomicBool>) -> Canceller {
		Canceller {
			cancelled,
		}
	}
	/// Cancel the context.
	pub fn cancel(&self) {
		self.cancelled.store(true, Ordering::Relaxed);
	}
}
