use super::*;

/// The watch on an inline native call in the simulation mode: a thread that
/// interrupts the interpreter's thread once, if the call outlasts `block=`.
pub(crate) enum Watch {
    Running {
        stop: std::sync::mpsc::Sender<()>,
        watcher: std::thread::JoinHandle<bool>,
    },
    #[cfg(test)]
    Completed(bool),
}

#[cfg(test)]
thread_local! {
    static TEST_RESULT: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Supplies the logical watch outcome for native-call functional controls.
#[cfg(test)]
pub(crate) fn set_test_result(result: Option<bool>) {
    TEST_RESULT.with(|slot| slot.set(result));
}

impl Interp {
    /// Starts the watch on an inline native call, in the simulation mode.
    pub(crate) fn sim_watch_native(&self) -> Option<Watch> {
        let bound = self.sim_block_bound()?;
        #[cfg(test)]
        if let Some(fired) = TEST_RESULT.with(std::cell::Cell::get) {
            return Some(Watch::Completed(fired));
        }
        let target = crate::signal::Interrupter::here();
        let (stop, stopped) = std::sync::mpsc::channel();
        let watcher = std::thread::spawn(move || {
            crate::signal::block();
            let timed_out = matches!(
                stopped.recv_timeout(bound),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            );
            if timed_out {
                target.interrupt();
                let _ = stopped.recv();
            }
            timed_out
        });
        Some(Watch::Running { stop, watcher })
    }

    /// Ends `watch` once its call has returned; a call that outlasted
    /// `block=` is refused at the next boundary.
    pub(crate) fn sim_end_watch(&mut self, watch: Watch) {
        let fired = match watch {
            Watch::Running { stop, watcher } => {
                let _ = stop.send(());
                watcher.join().unwrap_or(true)
            }
            #[cfg(test)]
            Watch::Completed(fired) => fired,
        };
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.blocked_native |= fired;
        }
    }
}
