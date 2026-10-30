use crate::config::Language;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    time::SystemTime,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PetState {
    #[default]
    Idle,
    Working,
    Input,
    Ready,
    Failed,
    Offline,
}
impl PetState {
    pub const ALL: [Self; 6] = [
        Self::Idle,
        Self::Working,
        Self::Input,
        Self::Ready,
        Self::Failed,
        Self::Offline,
    ];
    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Idle => [187, 154, 247],
            Self::Working => [122, 162, 247],
            Self::Input => [224, 175, 104],
            Self::Ready => [158, 206, 106],
            Self::Failed => [247, 118, 142],
            Self::Offline => [86, 95, 137],
        }
    }
    pub fn label(self, lang: Language) -> &'static str {
        match self {
            Self::Idle => lang.text("CODEX IDLE", "CODEX EN ESPERA"),
            Self::Working => lang.text("CODEX WORKING", "CODEX TRABAJANDO"),
            Self::Input => lang.text("NEEDS ATTENTION", "NECESITA ATENCIÓN"),
            Self::Ready => lang.text("TASK COMPLETE", "TAREA TERMINADA"),
            Self::Failed => lang.text("CODEX BLOCKED", "CODEX BLOQUEADO"),
            Self::Offline => lang.text("CODEX OFFLINE", "CODEX DESCONECTADO"),
        }
    }
}

#[derive(Default, Debug)]
pub struct SessionState {
    pub active: bool,
    pub terminal: PetState,
    pub last_event: Option<SystemTime>,
    pending: HashMap<String, WaitKind>,
    calls: HashSet<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitKind {
    Input,
    AsyncInput,
    Approval,
}

#[derive(Deserialize)]
struct Record<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    timestamp: Option<&'a str>,
    #[serde(borrow)]
    payload: &'a RawValue,
}
#[derive(Deserialize)]
struct Payload<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    name: Option<&'a str>,
    call_id: Option<&'a str>,
    request_id: Option<&'a str>,
    #[serde(borrow)]
    arguments: Option<&'a RawValue>,
}
impl SessionState {
    pub fn apply_approval_requests(&mut self, pending: &HashSet<String>) {
        for id in pending {
            if self.active && self.calls.contains(id) {
                self.pending.insert(id.clone(), WaitKind::Approval);
            }
        }
    }
    pub fn resolve_approvals(&mut self, resolved: &HashSet<String>) {
        self.pending
            .retain(|id, kind| *kind != WaitKind::Approval || !resolved.contains(id));
    }
    pub fn state(&self, now: SystemTime) -> PetState {
        if !self.pending.is_empty() {
            PetState::Input
        } else if self.active {
            PetState::Working
        } else if self.terminal == PetState::Ready
            && self
                .last_event
                .is_some_and(|time| now.duration_since(time).unwrap_or_default().as_secs() >= 600)
        {
            PetState::Idle
        } else {
            self.terminal
        }
    }
    pub fn apply_line(&mut self, line: &[u8]) {
        let line = line.strip_prefix(&[239, 187, 191]).unwrap_or(line);
        let Ok(record) = serde_json::from_slice::<Record<'_>>(line) else {
            return;
        };
        if !matches!(record.kind, "event_msg" | "response_item") {
            return;
        }
        let Ok(p) = serde_json::from_str::<Payload<'_>>(record.payload.get()) else {
            return;
        };
        let time = record
            .timestamp
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| SystemTime::from(t.with_timezone(&Utc)));
        let id = p.call_id.or(p.request_id).unwrap_or("unidentified");
        match p.kind {
            "task_started" | "turn_started" => {
                self.active = true;
                self.pending.clear();
                self.calls.clear();
                self.terminal = PetState::Working;
            }
            "task_complete" | "turn_complete" | "turn_completed" => {
                self.active = false;
                self.pending.clear();
                self.terminal = PetState::Ready;
                self.calls.clear();
                self.last_event = Some(time.unwrap_or_else(SystemTime::now));
            }
            "turn_aborted" | "error" | "failed" => {
                self.active = false;
                self.pending.clear();
                self.terminal = PetState::Failed;
                self.calls.clear();
                self.last_event = Some(time.unwrap_or_else(SystemTime::now));
            }
            "user_message" | "user_input" => {
                self.pending.clear();
            }
            "approval_response" | "elicitation_response" => {
                self.pending.remove(id);
            }
            "function_call" | "custom_tool_call" if self.active => {
                self.calls.insert(id.to_owned());
                let name = p.name.unwrap_or_default();
                if name.contains("request_user_input")
                    || name.contains("approval")
                    || name.contains("elicitation")
                {
                    self.pending.insert(
                        id.to_owned(),
                        if name.contains("request_user_input_async") {
                            WaitKind::AsyncInput
                        } else {
                            WaitKind::Input
                        },
                    );
                } else if p.arguments.is_some_and(escalated) {
                    // Remains pending until this exact call returns, never another tool's output.
                    self.pending.insert(id.to_owned(), WaitKind::Approval);
                }
            }
            "function_call_output" | "custom_tool_call_output" => {
                self.calls.remove(id);
                if self
                    .pending
                    .get(id)
                    .is_some_and(|kind| *kind != WaitKind::AsyncInput)
                {
                    self.pending.remove(id);
                }
            }
            kind if self.active
                && (kind.contains("approval")
                    || kind.contains("elicitation")
                    || kind == "request_user_input") =>
            {
                self.pending.insert(id.to_owned(), WaitKind::Input);
            }
            _ => {}
        }
    }
}

fn escalated(raw: &RawValue) -> bool {
    #[derive(Deserialize)]
    struct Args<'a> {
        sandbox_permissions: Option<&'a str>,
    }
    // Function arguments in Codex response items are normally a JSON-encoded string.
    if raw.get().starts_with('"') {
        serde_json::from_str::<String>(raw.get())
            .ok()
            .is_some_and(|json| {
                serde_json::from_str::<Args<'_>>(&json)
                    .ok()
                    .is_some_and(|a| a.sandbox_permissions == Some("require_escalated"))
            })
    } else {
        serde_json::from_str::<Args<'_>>(raw.get())
            .ok()
            .is_some_and(|a| a.sandbox_permissions == Some("require_escalated"))
    }
}

#[derive(Default)]
pub struct SessionReader {
    pub state: SessionState,
    offset: u64,
    pending: Vec<u8>,
}
impl SessionReader {
    pub fn poll(&mut self, path: &Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        if file.metadata()?.len() < self.offset {
            *self = Self::default();
        }
        file.seek(SeekFrom::Start(self.offset))?;
        // Bound a single poll so a growing or very large log cannot monopolize the worker.
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024).read_to_end(&mut bytes)?;
        self.offset += bytes.len() as u64;
        self.pending.extend_from_slice(&bytes);
        let mut consumed = 0;
        for (i, b) in self.pending.iter().enumerate() {
            if *b == b'\n' {
                self.state.apply_line(&self.pending[consumed..i]);
                consumed = i + 1;
            }
        }
        self.pending.drain(..consumed);
        if self.pending.capacity() > 1024 * 1024 && self.pending.len() < 65536 {
            self.pending.shrink_to(65536);
        }
        // Preserve byte fragments across polls, including incomplete UTF-8 and JSON records.
        Ok(())
    }
}

pub fn aggregate<'a>(
    states: impl Iterator<Item = &'a SessionState>,
    running: bool,
    now: SystemTime,
) -> PetState {
    if !running {
        return PetState::Offline;
    }
    let states: Vec<_> = states.collect();
    if states.iter().any(|s| s.state(now) == PetState::Input) {
        return PetState::Input;
    }
    if states.iter().any(|s| s.state(now) == PetState::Working) {
        return PetState::Working;
    }
    states
        .into_iter()
        .max_by_key(|s| s.last_event)
        .map(|s| s.state(now))
        .unwrap_or(PetState::Idle)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(kind: &str) -> Vec<u8> {
        format!(r#"{{"type":"event_msg","payload":{{"type":"{kind}"}}}}"#).into_bytes()
    }
    #[test]
    fn unrelated_output_does_not_resolve_input() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"A"}}"#);
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call_output","call_id":"B","output":"ok"}}"#);
        assert_eq!(s.state(SystemTime::now()), PetState::Input);
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call_output","call_id":"A","output":"yes"}}"#);
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn asynchronous_question_survives_tool_acknowledgement() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call","name":"request_user_input_async","call_id":"A"}}"#);
        s.apply_line(
            br#"{"type":"response_item","payload":{"type":"function_call_output","call_id":"A"}}"#,
        );
        assert_eq!(s.state(SystemTime::now()), PetState::Input);
        s.apply_line(&event("user_message"));
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn reordered_properties_and_nested_tool_results_are_safe() {
        let mut s = SessionState::default();
        s.apply_line(br#"{"payload":{"turn_id":"x","type":"task_started"},"type":"event_msg"}"#);
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call_output","output":{"payload":{"type":"task_complete"}}}}"#);
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn all_sessions_participate_in_priority() {
        let mut a = SessionState::default();
        a.apply_line(&event("task_started"));
        let mut b = SessionState::default();
        b.apply_line(&event("task_complete"));
        assert_eq!(
            aggregate([&a, &b].into_iter(), true, SystemTime::now()),
            PetState::Working
        );
        assert_eq!(
            aggregate([&a, &b].into_iter(), false, SystemTime::now()),
            PetState::Offline
        );
    }
    #[test]
    fn completed_state_expires_using_event_timestamp() {
        let mut s = SessionState::default();
        s.apply_line(br#"{"type":"event_msg","timestamp":"2020-01-01T00:00:00Z","payload":{"type":"task_complete"}}"#);
        assert_eq!(s.state(SystemTime::now()), PetState::Idle);
    }
    #[test]
    fn escalated_call_waits_for_its_own_output() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call","name":"exec_command","call_id":"A","arguments":"{\"sandbox_permissions\":\"require_escalated\"}"}}"#);
        assert_eq!(s.state(SystemTime::now()), PetState::Input);
        s.apply_line(
            br#"{"type":"response_item","payload":{"type":"function_call_output","call_id":"A"}}"#,
        );
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn approval_grant_resumes_work_before_command_finishes() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call","name":"exec_command","call_id":"A","arguments":{"sandbox_permissions":"require_escalated"}}}"#);
        s.resolve_approvals(&HashSet::from(["B".to_owned()]));
        assert_eq!(s.state(SystemTime::now()), PetState::Input);
        s.resolve_approvals(&HashSet::from(["A".to_owned()]));
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn external_approval_only_affects_a_matching_active_call() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        s.apply_line(br#"{"type":"response_item","payload":{"type":"function_call","name":"functions.exec","call_id":"wrapper-A"}}"#);
        s.apply_approval_requests(&HashSet::from(["other".into()]));
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
        s.apply_approval_requests(&HashSet::from(["wrapper-A".into()]));
        assert_eq!(s.state(SystemTime::now()), PetState::Input);
        s.resolve_approvals(&HashSet::from(["wrapper-A".into()]));
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
    #[test]
    fn incremental_reader_retains_partial_records_and_handles_truncation() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "codexpet-reader-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut reader = SessionReader::default();
        let line = br#"{"type":"event_msg","payload":{"extra":"caf\u00e9","type":"task_started"}}"#;
        file.write_all(&line[..30]).unwrap();
        file.flush().unwrap();
        reader.poll(&path).unwrap();
        assert_eq!(reader.state.state(SystemTime::now()), PetState::Idle);
        file.write_all(&line[30..]).unwrap();
        file.write_all(b"\n").unwrap();
        file.flush().unwrap();
        reader.poll(&path).unwrap();
        assert_eq!(reader.state.state(SystemTime::now()), PetState::Working);
        drop(file);
        std::fs::write(&path, [event("task_complete"), b"\n".to_vec()].concat()).unwrap();
        reader.poll(&path).unwrap();
        assert_eq!(reader.state.state(SystemTime::now()), PetState::Ready);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn multi_megabyte_tool_output_does_not_change_turn_state() {
        let mut s = SessionState::default();
        s.apply_line(&event("task_started"));
        let mut output =
            br#"{"type":"response_item","payload":{"type":"custom_tool_call_output","output":""#
                .to_vec();
        output.extend(std::iter::repeat_n(b'x', 2_000_000));
        output.extend_from_slice(br#""}}"#);
        s.apply_line(&output);
        assert_eq!(s.state(SystemTime::now()), PetState::Working);
    }
}
