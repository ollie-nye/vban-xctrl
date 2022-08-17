use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

// Shamelessly stolen from https://gist.github.com/NoraCodes/e6d40782b05dc8ac40faf3a0405debd3

#[derive(Clone)]
pub struct WorkQueue<T: Send + Clone> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send + Clone> WorkQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_work(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();
        if let Ok(mut queue) = maybe_queue {
            queue.pop_front()
        } else {
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }

    pub fn add_work(&self, work: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work);

            queue.len()
        } else {
            panic!("WorkQueue::add_work() tried to lock a poisoned mutex");
        }
    }
}
