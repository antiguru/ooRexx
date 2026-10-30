use super::*;

/// Spawns `command`, writes `bytes` to its standard input, closes it, and waits.
pub fn write_and_wait(command: &mut Command, bytes: &[u8], path: &Path) -> std::process::Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn for {}: {e}", path.display()));
    write_stdin(&mut child, bytes);
    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("failed to wait for {}: {e}", path.display()))
}

pub(super) fn write_stdin(child: &mut Child, bytes: &[u8]) {
    use std::io::Write as _;
    let mut sink = child.stdin.take().expect("stdin was requested as a pipe");
    let bytes = bytes.to_vec();
    // A descendant may retain stdin after the direct child exits, so the
    // writer must not be joined by the bounded wait.
    thread::spawn(move || {
        let _ = sink.write_all(&bytes);
    });
}

/// Waits for `child` under `deadline`, reading both output pipes
/// concurrently, and returns what it produced along with how it ended.
pub(super) fn wait_with_deadline(
    mut child: Child,
    path: &Path,
    deadline: Duration,
) -> (Vec<u8>, Vec<u8>, Termination) {
    let mut stdout_pipe = child.stdout.take().expect("stdout was requested as a pipe");
    let mut stderr_pipe = child.stderr.take().expect("stderr was requested as a pipe");
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        let _ = stdout_tx.send(buf);
    });
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        let _ = stderr_tx.send(buf);
    });

    let start = Instant::now();
    let (mut code, mut deadline_exceeded) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) => {
                if start.elapsed() >= deadline {
                    // The process is still running past its deadline: kill
                    // it, then bind the status `wait()` actually returns
                    // rather than assuming `None`. A process that exited in
                    // the gap between the last `try_wait` and this check is
                    // killed as a no-op -- the signal lands on an
                    // already-dead process -- and `wait()` still reports its
                    // real status, so binding it here turns what would
                    // otherwise be a flaky `TimedOut` (and the panic
                    // `expect_exit_code` gives one) into the `Exited` this
                    // run actually earned.
                    report_timeout(path, "child wait", deadline, start.elapsed());
                    let _ = child.kill();
                    let status = child.wait().ok();
                    break (status.and_then(|s| s.code()), true);
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(e) => panic!("failed to wait for {}: {e}", path.display()),
        }
    };

    // Bounded independently of the wait above -- see "Why the reader
    // threads report over a channel" -- so a fast-exiting direct child with
    // a slow grandchild still has this whole deadline to finish reading in,
    // and a read that has already outlasted one deadline is not handed a
    // second one on top by chaining off the first one's remaining time. The
    // two channels share one clock rather than each getting a fresh
    // deadline, so the pair together are bounded by it once, not
    // twice.
    let read_start = Instant::now();
    let read_deadline = read_start + deadline;
    let stdout_result =
        stdout_rx.recv_timeout(read_deadline.saturating_duration_since(Instant::now()));
    let stderr_result =
        stderr_rx.recv_timeout(read_deadline.saturating_duration_since(Instant::now()));

    // A disconnected channel means the sender end was dropped without
    // sending -- the reader thread panicked before it could report its
    // buffer -- which `recv_timeout` distinguishes from an ordinary
    // `Timeout` and this function does too, rather than reading both the
    // same way. Folding it into a `TimedOut` classification instead would
    // trade a named harness bug for a silent misclassification.
    if matches!(stdout_result, Err(mpsc::RecvTimeoutError::Disconnected)) {
        panic!("stdout reader thread panicked for {}", path.display());
    }
    if matches!(stderr_result, Err(mpsc::RecvTimeoutError::Disconnected)) {
        panic!("stderr reader thread panicked for {}", path.display());
    }

    let (stdout, stderr) = match (stdout_result, stderr_result) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        _ => {
            report_timeout(path, "output drain", deadline, read_start.elapsed());
            // At least one side is still blocked, almost always behind a
            // descriptor its direct child handed to a process of its own.
            // The transcript is not trustworthy either way -- a stdout that
            // did arrive on time proves nothing about a stderr that did not,
            // and vice versa -- so this run counts as a non-finish and both
            // channels read empty. Free to do: a non-finish's bytes are a
            // structural failure, never a comparison, so nothing downstream
            // reads them.
            if matches!(
                classify_termination(code, deadline_exceeded),
                Termination::Exited(_)
            ) {
                code = None;
                deadline_exceeded = true;
            }
            (Vec::new(), Vec::new())
        }
    };

    (
        stdout,
        stderr,
        classify_termination(code, deadline_exceeded),
    )
}

fn timeout_context(
    path: &Path,
    phase: &str,
    deadline: Duration,
    elapsed: Duration,
    load: Result<&[u8], &std::io::Error>,
) -> String {
    let load = match load {
        Ok(bytes) => String::from_utf8_lossy(bytes).trim().to_string(),
        Err(error) => format!("unavailable ({error})"),
    };
    format!(
        "oracle timeout: probe={} phase={phase} configured={deadline:?} elapsed={elapsed:?} loadavg={load}",
        path.display()
    )
}

fn report_timeout(path: &Path, phase: &str, deadline: Duration, elapsed: Duration) {
    let load = std::fs::read("/proc/loadavg");
    eprintln!(
        "{}",
        timeout_context(path, phase, deadline, elapsed, load.as_deref())
    );
}

#[cfg(test)]
mod tests;
