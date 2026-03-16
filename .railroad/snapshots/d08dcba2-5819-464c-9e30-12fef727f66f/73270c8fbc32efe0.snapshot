use std::path::Path;

use crate::trace::logger::log_trace;
use crate::types::{HookInput, Policy, TraceEntry};

/// Handle a SessionStart event.
/// Logs the session initialization.
pub fn handle(input: &HookInput, policy: &Policy) {
    if !policy.trace.enabled {
        return;
    }

    let trace_dir = Path::new(&input.cwd).join(&policy.trace.directory);

    let entry = TraceEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_id: input.session_id.clone(),
        event: "SessionStart".to_string(),
        tool: "-".to_string(),
        input_summary: format!("Session started in {}", input.cwd),
        decision: "allow".to_string(),
        rule: None,
        duration_ms: 0,
    };

    if let Err(e) = log_trace(&trace_dir, &input.session_id, &entry) {
        eprintln!("railyard: trace warning: {}", e);
    }
}
