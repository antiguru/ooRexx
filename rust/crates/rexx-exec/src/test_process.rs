use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Runs the named unit test alone with bounded virtual address space and an
/// open, empty stdin pipe. The resident cap is inherited from the test driver.
pub(crate) fn isolated(name: &str) -> bool {
    const CHILD: &str = "REXX_POOL_FAILURE_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        let pool = crate::scheduler::Pool::new(1 << 46, 1);
        match pool.reserve() {
            Err(_) => assert_eq!(pool.threads(), 0),
            Ok(Some(worker)) => {
                worker.run(Box::new(|| {}));
                panic!("huge-stack seam unexpectedly spawned a worker");
            }
            Ok(None) => panic!("huge-stack seam selected configured inline mode"),
        }
        return false;
    }
    let executable = std::env::current_exe().expect("test executable");
    let mut child = Command::new("python3")
        .args(["-c", "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(8<<30,8<<30)); os.execv(sys.argv[1],sys.argv[1:])"])
        .arg(executable).args(["--exact", name, "--nocapture"])
        .env(CHILD, name).stdin(Stdio::piped()).spawn().expect("isolated test");
    let writer = child.stdin.take().expect("open empty pipe");
    let deadline = Instant::now() + Duration::from_secs(25);
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed out child");
            child.wait().expect("reap timed out child");
            panic!("isolated test blocked, possibly reading stdin inline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);
    assert!(status.success(), "isolated test failed: {status}");
    true
}
