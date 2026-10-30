use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(super) struct OwnedOrphan {
    directory: PathBuf,
    finished: bool,
}

fn identity(stat: &str) -> (i32, &str, &str) {
    let pid = stat
        .split_once(' ')
        .expect("proc PID")
        .0
        .parse()
        .expect("numeric PID");
    let fields = stat.rsplit_once(") ").expect("proc command").1;
    let mut fields = fields.split_whitespace();
    let state = fields.next().expect("proc state");
    let start = fields.nth(18).expect("proc start time");
    (pid, start, state)
}

impl OwnedOrphan {
    pub(super) fn new(directory: &Path) -> Self {
        Self {
            directory: directory.canonicalize().expect("fixture cwd"),
            finished: false,
        }
    }

    pub(super) fn finish(mut self) {
        self.stop().expect("stop the owned orphan");
        self.finished = true;
    }

    fn stop(&self) -> Result<(), String> {
        let recorded = fs::read_to_string(self.directory.join("orphan.stat"))
            .map_err(|error| error.to_string())?;
        let (pid, start, _) = identity(&recorded);
        let proc = PathBuf::from(format!("/proc/{pid}"));
        let pid = rustix::process::Pid::from_raw(pid).ok_or("invalid orphan PID")?;
        let pidfd = match rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::SRCH) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let Ok(current) = fs::read_to_string(proc.join("stat")) else {
            return Ok(());
        };
        let (_, current_start, state) = identity(&current);
        if current_start != start || state == "Z" {
            return Ok(());
        }
        if fs::read_link(proc.join("cwd")).ok().as_ref() != Some(&self.directory)
            || fs::read(proc.join("cmdline")).ok().as_deref() != Some(b"sleep\x0030\x00")
        {
            return Err(format!("owned orphan {pid} changed identity"));
        }
        eprintln!("stopping owned pipe-holder PID {pid}, start time {start}, sleep limit 30s");
        if let Err(error) =
            rustix::process::pidfd_send_signal(&pidfd, rustix::process::Signal::KILL)
        {
            if error != rustix::io::Errno::SRCH {
                return Err(error.to_string());
            }
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while let Ok(current) = fs::read_to_string(proc.join("stat")) {
            let (_, current_start, state) = identity(&current);
            // A reparented grandchild cannot be waited by this process.
            if current_start != start || state == "Z" {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(format!("owned orphan {pid} still runs"));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }
}

impl Drop for OwnedOrphan {
    fn drop(&mut self) {
        if !self.finished && self.directory.join("orphan.stat").is_file() {
            if let Err(error) = self.stop() {
                eprintln!("owned orphan cleanup failed: {error}");
            }
        }
    }
}
