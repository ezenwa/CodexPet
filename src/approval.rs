//! Optional compatibility with Codex's local approval audit log. SQL is read-only,
//! bounded by row IDs, and resolutions are matched to the exact tool call ID.
use std::{
    collections::{HashSet, VecDeque},
    ffi::{CStr, CString, c_char, c_int, c_void},
    path::Path,
    ptr,
};

#[link(name = "winsqlite3")]
unsafe extern "C" {
    fn sqlite3_open_v2(
        path: *const c_char,
        db: *mut *mut c_void,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(db: *mut c_void) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut c_void,
        sql: *const c_char,
        bytes: c_int,
        stmt: *mut *mut c_void,
        tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_finalize(stmt: *mut c_void) -> c_int;
    fn sqlite3_step(stmt: *mut c_void) -> c_int;
    fn sqlite3_column_int64(stmt: *mut c_void, column: c_int) -> i64;
    fn sqlite3_column_text(stmt: *mut c_void, column: c_int) -> *const u8;
    fn sqlite3_busy_timeout(db: *mut c_void, ms: c_int) -> c_int;
}
struct Database(*mut c_void);
impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            sqlite3_close(self.0);
        }
    }
}
struct Statement(*mut c_void);
impl Drop for Statement {
    fn drop(&mut self) {
        unsafe {
            sqlite3_finalize(self.0);
        }
    }
}
impl Database {
    fn open(path: &Path) -> Option<Self> {
        let path = CString::new(path.to_str()?).ok()?;
        let mut raw = ptr::null_mut();
        let result = unsafe { sqlite3_open_v2(path.as_ptr(), &mut raw, 1, ptr::null()) };
        if result != 0 {
            if !raw.is_null() {
                unsafe {
                    sqlite3_close(raw);
                }
            }
            return None;
        }
        unsafe {
            sqlite3_busy_timeout(raw, 30);
        }
        Some(Self(raw))
    }
    fn query(&self, sql: &str) -> Option<Statement> {
        let sql = CString::new(sql).ok()?;
        let mut raw = ptr::null_mut();
        if unsafe { sqlite3_prepare_v2(self.0, sql.as_ptr(), -1, &mut raw, ptr::null_mut()) } != 0 {
            return None;
        }
        Some(Statement(raw))
    }
}

#[derive(Default)]
pub struct ApprovalTracker {
    last_id: i64,
    pub resolved: HashSet<String>,
    pub pending: HashSet<String>,
    order: VecDeque<String>,
}
impl ApprovalTracker {
    pub fn poll(&mut self, path: &Path) {
        let Some(db) = Database::open(path) else {
            return;
        };
        let Some(max) = db.query("SELECT MAX(id) FROM logs") else {
            return;
        };
        if unsafe { sqlite3_step(max.0) } != 100 {
            return;
        }
        let current = unsafe { sqlite3_column_int64(max.0, 0) };
        if current == self.last_id {
            return;
        }
        if current < self.last_id {
            *self = Self::default();
        }
        let since = self.last_id.max(current.saturating_sub(8192));
        let sql = format!(
            "SELECT feedback_log_body FROM logs WHERE id>{since} AND id<={current} AND ((target='codex_core::session::handlers' AND feedback_log_body LIKE '%op: ExecApproval %') OR (target='codex_core::stream_events_utils' AND feedback_log_body LIKE '%handle_output_item_done: ToolCall:%' AND feedback_log_body LIKE '%sandbox_permissions%require_escalated%')) ORDER BY id"
        );
        let Some(statement) = db.query(&sql) else {
            return;
        };
        loop {
            match unsafe { sqlite3_step(statement.0) } {
                100 => {}
                101 => break,
                _ => return,
            }
            let raw = unsafe { sqlite3_column_text(statement.0, 0) };
            if raw.is_null() {
                continue;
            }
            let Ok(body) = unsafe { CStr::from_ptr(raw.cast()) }.to_str() else {
                continue;
            };
            if let Some(id) = resolution_id(body)
                && self.resolved.insert(id.to_owned())
            {
                self.pending.remove(id);
                self.order.push_back(id.to_owned());
            } else if let Some(id) = request_id(body)
                && !self.resolved.contains(id)
                && self.pending.insert(id.to_owned())
            {
                self.order.push_back(id.to_owned());
            }
        }
        self.last_id = current;
        while self.order.len() > 4096 {
            if let Some(id) = self.order.pop_front() {
                self.resolved.remove(&id);
                self.pending.remove(&id);
            }
        }
    }
}
fn request_id(body: &str) -> Option<&str> {
    let record = body.split_once("handle_output_item_done: ToolCall:")?.1;
    if !record.contains("sandbox_permissions") || !record.contains("require_escalated") {
        return None;
    }
    let id = record.split_once("call_id: \"")?.1.split_once('"')?.0;
    if id.is_empty() || id.len() > 256 {
        return None;
    }
    Some(id)
}
fn resolution_id(body: &str) -> Option<&str> {
    let record = body.split_once("op: ExecApproval {")?.1;
    let id = record.split_once("id: \"")?.1.split_once('"')?.0;
    if id.is_empty() || id.len() > 256 {
        return None;
    }
    Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolution_is_scoped_to_an_explicit_approval() {
        assert_eq!(
            resolution_id("op: ExecApproval { id: \"call-A\", decision: Approved }"),
            Some("call-A")
        );
        assert_eq!(resolution_id("tool call completed call-A"), None);
    }
    #[test]
    fn wrapped_escalation_keeps_its_call_identity() {
        assert_eq!(
            request_id(
                r#"handle_output_item_done: ToolCall: FunctionCall { arguments: "{code: sandbox_permissions require_escalated}", call_id: "wrapper-A" }"#
            ),
            Some("wrapper-A")
        );
        assert_eq!(request_id("tool call completed wrapper-A"), None);
    }
}
