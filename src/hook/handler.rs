use std::io::Read;
use std::path::Path;

use crate::hook::{post_tool, pre_tool, session};
use crate::policy::loader::load_policy_or_defaults;
use crate::types::HookInput;

/// Main hook entry point. Reads JSON from stdin, dispatches to the right handler.
pub fn run(event: &str) -> i32 {
    // Read stdin
    let mut input_str = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input_str) {
        eprintln!("railyard: failed to read stdin: {}", e);
        return 0; // Don't block on read errors
    }

    // Parse input
    let input: HookInput = match serde_json::from_str(&input_str) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("railyard: failed to parse hook input: {}", e);
            return 0; // Don't block on parse errors
        }
    };

    // Load policy
    let cwd = Path::new(&input.cwd);
    let policy = load_policy_or_defaults(cwd);

    // Dispatch
    match event {
        "PreToolUse" => {
            let output = pre_tool::handle(&input, &policy);
            // Write output to stdout
            if let Ok(json) = serde_json::to_string(&output) {
                println!("{}", json);
            }
            // If we denied, output was already written — exit 0
            // (we use permissionDecision: "deny" in the JSON, not exit code 2)
            0
        }
        "PostToolUse" => {
            post_tool::handle(&input, &policy);
            0
        }
        "SessionStart" => {
            session::handle(&input, &policy);
            0
        }
        _ => {
            eprintln!("railyard: unknown event: {}", event);
            0
        }
    }
}
