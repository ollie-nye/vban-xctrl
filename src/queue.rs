use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

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

pub struct SyncFlagTx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagTx {
    // This function will be used by the controller thread to tell the worker
    // threads about the end of computation.

    /// Sets the interior value of the SyncFlagTx which will be read by any
    /// SyncFlagRx that exist for this SyncFlag.
    ///
    /// # Errors
    /// If the underlying mutex is poisoned this may return an error.
    pub fn set(&mut self, state: bool) -> Result<(), ()> {
        if let Ok(mut v) = self.inner.lock() {
            // The * (deref operator) means assigning to what's inside the
            // MutexGuard, not the guard itself (which would be silly)
            *v = state;
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Clone)]
pub struct SyncFlagRx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagRx {
    // This function will be used by the worker threads to check if they should
    // stop looking for work to do.

    /// Gets the interior state of the SyncFlagRx to whatever the corresponding
    /// SyncFlagTx last set it to.
    ///
    /// # Errors
    /// If the underlying mutex is poisoned this might return an error.
    pub fn get(&self) -> Result<bool, ()> {
        if let Ok(v) = self.inner.lock() {
            // Deref the MutexGuard to get at the bool inside
            Ok(*v)
        } else {
            Err(())
        }
    }
}

pub fn new_syncflag(initial_state: bool) -> (SyncFlagTx, SyncFlagRx) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagTx {
        inner: state.clone(),
    };
    let rx = SyncFlagRx {
        inner: state.clone(),
    };

    return (tx, rx);
}
