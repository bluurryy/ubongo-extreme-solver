use std::cell::Cell;
use std::rc::Rc;

use futures::future::{abortable, AbortHandle};
use futures::Future;
use sycamore::prelude::*;
use tracing::debug;

pub struct Task {
    abort_handle: Cell<Option<AbortHandle>>,
    is_running: Signal<bool>,
}

impl Task {
    pub fn new() -> Self {
        Self {
            abort_handle: Cell::new(None),
            is_running: create_signal(false),
        }
    }

    pub fn abort(&self) {
        if let Some(abort_handle) = self.abort_handle.take() {
            abort_handle.abort();
        }
    }

    pub fn is_running(&self) -> ReadSignal<bool> {
        *self.is_running
    }

    pub fn spawn_local(&self, f: impl Future<Output = ()> + 'static) {
        self.abort();
        let (abortable, handle) = abortable(f);
        self.abort_handle.set(Some(handle));

        let is_running = self.is_running;
        is_running.set(true);

        sycamore::futures::spawn_local_scoped(async move {
            if abortable.await.is_err() {
                debug!("Future was aborted!");
            }

            is_running.set(false);
        });
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.abort();
    }
}

pub fn create_task() -> Rc<Task> {
    Rc::new(Task::new())
}
