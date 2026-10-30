use crate::state::{PetState, SessionReader, aggregate};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub state: PetState,
    pub running: bool,
}

pub fn start(
    home: PathBuf,
    sender: Sender<Snapshot>,
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sessions = HashMap::<PathBuf, SessionReader>::new();
        let mut files = Vec::new();
        let mut next_discovery = SystemTime::UNIX_EPOCH;
        let mut previous_running = false;
        #[cfg(windows)]
        let mut approvals = crate::approval::ApprovalTracker::default();
        while !stop.load(Ordering::Relaxed) {
            let now = SystemTime::now();
            #[cfg(windows)]
            let processes = crate::platform::codex_processes();
            #[cfg(not(windows))]
            let processes: Vec<SystemTime> = Vec::new();
            let running = !processes.is_empty();
            if running && (!previous_running || now >= next_discovery) {
                files.clear();
                discover(&home.join("sessions"), &mut files);
                let earliest = processes.iter().min().copied().unwrap_or(now);
                // Old unfinished turns from exited Codex processes must not survive a restart.
                let cutoff = earliest
                    .checked_sub(Duration::from_secs(2))
                    .unwrap_or(earliest);
                files.retain(|p| {
                    fs::metadata(p)
                        .and_then(|m| m.modified())
                        .is_ok_and(|t| t >= cutoff)
                });
                let current: HashSet<_> = files.iter().cloned().collect();
                sessions.retain(|p, _| current.contains(p));
                next_discovery = now + Duration::from_secs(5);
            }
            if running {
                for path in &files {
                    let _ = sessions.entry(path.clone()).or_default().poll(path);
                }
                #[cfg(windows)]
                {
                    approvals.poll(&home.join("logs_2.sqlite"));
                    for reader in sessions.values_mut() {
                        reader.state.resolve_approvals(&approvals.resolved);
                        reader.state.apply_approval_requests(&approvals.pending);
                    }
                }
            } else {
                sessions.clear();
                files.clear();
            }
            let state = aggregate(sessions.values().map(|reader| &reader.state), running, now);
            if sender.send(Snapshot { state, running }).is_err() {
                break;
            }
            previous_running = running;
            for _ in 0..10 {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    })
}

fn discover(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            discover(&entry.path(), files);
        } else if kind.is_file() && entry.path().extension().is_some_and(|e| e == "jsonl") {
            files.push(entry.path());
        }
    }
}
