use super::*;

/// The watch on an inline native call in the simulation mode: a thread that
/// interrupts the interpreter's thread once, if the call outlasts `block=`.
pub(crate) struct Watch {
    stop: std::sync::mpsc::Sender<()>,
    watcher: std::thread::JoinHandle<bool>,
}

impl Interp {
    /// Starts the watch on an inline native call, in the simulation mode.
    pub(crate) fn sim_watch_native(&self) -> Option<Watch> {
        let bound = self.sim_block_bound()?;
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
        Some(Watch { stop, watcher })
    }

    /// Ends `watch` once its call has returned; a call that outlasted
    /// `block=` is refused at the next boundary.
    pub(crate) fn sim_end_watch(&mut self, watch: Watch) {
        let _ = watch.stop.send(());
        let fired = watch.watcher.join().unwrap_or(true);
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.blocked_native |= fired;
        }
    }

}
