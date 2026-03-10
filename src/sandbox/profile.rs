use std::fs;
use std::path::Path;

use crate::sandbox::detect::{detect_sandbox, SandboxCapability};
use crate::sandbox::{linux, macos};
use crate::types::Policy;

/// Generate sandbox profiles from the policy's fence config.
/// Writes platform-appropriate profile to `.railyard/`.
pub fn generate_profiles(policy: &Policy, cwd: &str) -> Result<String, String> {
    let sandbox_dir = Path::new(cwd).join(".railyard");
    fs::create_dir_all(&sandbox_dir).map_err(|e| format!("create sandbox dir: {}", e))?;

    let capability = detect_sandbox();

    match capability {
        SandboxCapability::MacOsSandboxExec => {
            let profile = macos::generate_profile(&policy.fence, cwd);
            let profile_path = sandbox_dir.join("sandbox.sb");
            fs::write(&profile_path, &profile)
                .map_err(|e| format!("write sandbox profile: {}", e))?;

            Ok(format!(
                "Generated macOS sandbox profile: {}\n\
                 Usage: sandbox-exec -f {} -- sh -c \"your-command\"",
                profile_path.display(),
                profile_path.display()
            ))
        }

        SandboxCapability::LinuxLandlock { abi_version } => {
            // Generate both bwrap wrapper and Landlock snippet
            let bwrap_cmd = linux::generate_bwrap_command(&policy.fence, cwd);
            let bwrap_path = sandbox_dir.join("sandbox-bwrap.sh");
            let bwrap_script = format!(
                "#!/bin/sh\n# Railyard sandbox wrapper (bubblewrap)\n# Usage: .railyard/sandbox-bwrap.sh your-command\n\n{} \"$@\"\n",
                bwrap_cmd
            );
            fs::write(&bwrap_path, &bwrap_script)
                .map_err(|e| format!("write bwrap script: {}", e))?;

            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&bwrap_path, fs::Permissions::from_mode(0o755));
            }

            let landlock_code = linux::generate_landlock_snippet(&policy.fence, cwd);
            let landlock_path = sandbox_dir.join("sandbox-landlock.rs");
            fs::write(&landlock_path, &landlock_code)
                .map_err(|e| format!("write landlock snippet: {}", e))?;

            Ok(format!(
                "Generated Linux sandbox profiles (Landlock ABI v{}):\n\
                 \x20 Bubblewrap: {} -- your-command\n\
                 \x20 Landlock:   {} (Rust reference code)",
                abi_version,
                bwrap_path.display(),
                landlock_path.display()
            ))
        }

        SandboxCapability::None => Err(
            "No OS-level sandbox available on this platform.\n\
             macOS: requires sandbox-exec (built-in on all macOS versions)\n\
             Linux: requires kernel 5.13+ for Landlock"
                .to_string(),
        ),
    }
}

/// Launch Claude Code inside the OS sandbox.
/// This wraps the entire Claude Code process so ALL child processes
/// (shell commands, file access, network) are kernel-restricted.
pub fn launch_sandboxed_claude(policy: &Policy, cwd: &str, claude_args: &[String]) -> Result<i32, String> {
    let capability = detect_sandbox();

    match capability {
        SandboxCapability::MacOsSandboxExec => {
            let profile = macos::generate_claude_profile(&policy.fence, cwd);
            let profile_path = Path::new(cwd).join(".railyard/sandbox-claude.sb");
            fs::create_dir_all(profile_path.parent().unwrap())
                .map_err(|e| format!("create dir: {}", e))?;
            fs::write(&profile_path, &profile)
                .map_err(|e| format!("write profile: {}", e))?;

            // Find the claude binary
            let claude_bin = which_claude()?;

            let mut cmd = std::process::Command::new("sandbox-exec");
            cmd.arg("-f")
                .arg(&profile_path)
                .arg("--")
                .arg(&claude_bin);

            for arg in claude_args {
                cmd.arg(arg);
            }

            // On Unix, use exec() to replace our process entirely
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let err = cmd.exec();
                return Err(format!("exec sandbox-exec failed: {}", err));
            }

            #[cfg(not(unix))]
            {
                let status = cmd
                    .status()
                    .map_err(|e| format!("sandbox-exec failed: {}", e))?;
                Ok(status.code().unwrap_or(1))
            }
        }

        SandboxCapability::LinuxLandlock { .. } => {
            // Check for bwrap
            let bwrap_check = std::process::Command::new("which")
                .arg("bwrap")
                .output();

            if let Ok(output) = &bwrap_check {
                if output.status.success() {
                    let claude_bin = which_claude()?;
                    let home = dirs::home_dir()
                        .map(|h| h.display().to_string())
                        .unwrap_or_default();

                    let mut cmd = std::process::Command::new("bwrap");
                    // System paths (read-only)
                    cmd.args(["--ro-bind", "/usr", "/usr"]);
                    cmd.args(["--ro-bind", "/bin", "/bin"]);
                    cmd.args(["--ro-bind", "/lib", "/lib"]);
                    cmd.args(["--ro-bind", "/lib64", "/lib64"]);
                    cmd.args(["--ro-bind", "/sbin", "/sbin"]);
                    cmd.args(["--ro-bind", "/etc", "/etc"]);
                    // Dev and proc
                    cmd.args(["--dev", "/dev"]);
                    cmd.args(["--proc", "/proc"]);
                    cmd.args(["--tmpfs", "/tmp"]);
                    // Project dir (read-write)
                    cmd.args(["--bind", cwd, cwd]);
                    // Claude Code config
                    let claude_dir = format!("{}/.claude", home);
                    cmd.args(["--bind", &claude_dir, &claude_dir]);
                    // Toolchains (read-only)
                    let cargo_dir = format!("{}/.cargo", home);
                    cmd.args(["--ro-bind", &cargo_dir, &cargo_dir]);
                    // Shadow sensitive dirs
                    let ssh_dir = format!("{}/.ssh", home);
                    let aws_dir = format!("{}/.aws", home);
                    let gnupg_dir = format!("{}/.gnupg", home);
                    cmd.args(["--tmpfs", &ssh_dir]);
                    cmd.args(["--tmpfs", &aws_dir]);
                    cmd.args(["--tmpfs", &gnupg_dir]);

                    cmd.arg("--").arg(&claude_bin);
                    for arg in claude_args {
                        cmd.arg(arg);
                    }

                    #[cfg(unix)]
                    {
                        use std::os::unix::process::CommandExt;
                        let err = cmd.exec();
                        return Err(format!("exec bwrap failed: {}", err));
                    }

                    #[cfg(not(unix))]
                    {
                        let status = cmd.status()
                            .map_err(|e| format!("bwrap failed: {}", e))?;
                        return Ok(status.code().unwrap_or(1));
                    }
                }
            }

            Err("bubblewrap (bwrap) not found. Install: apt install bubblewrap".to_string())
        }

        SandboxCapability::None => {
            Err("No OS-level sandbox available on this platform.\n\
                 macOS: sandbox-exec is built-in\n\
                 Linux: requires kernel 5.13+ for Landlock, or install bubblewrap"
                .to_string())
        }
    }
}

/// Find the claude binary path.
fn which_claude() -> Result<String, String> {
    let output = std::process::Command::new("which")
        .arg("claude")
        .output()
        .map_err(|e| format!("failed to find claude: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Claude Code CLI not found. Install: https://docs.anthropic.com/en/docs/claude-code".to_string())
    }
}

/// Run a command inside the sandbox.
pub fn run_sandboxed(policy: &Policy, cwd: &str, command: &[String]) -> Result<i32, String> {
    let capability = detect_sandbox();
    let cmd_str = command.join(" ");

    match capability {
        SandboxCapability::MacOsSandboxExec => {
            // Generate profile to temp location
            let profile = macos::generate_profile(&policy.fence, cwd);
            let profile_path = Path::new(cwd).join(".railyard/sandbox.sb");
            fs::create_dir_all(profile_path.parent().unwrap())
                .map_err(|e| format!("create dir: {}", e))?;
            fs::write(&profile_path, &profile)
                .map_err(|e| format!("write profile: {}", e))?;

            let status = std::process::Command::new("sandbox-exec")
                .arg("-f")
                .arg(&profile_path)
                .arg("--")
                .arg("sh")
                .arg("-c")
                .arg(&cmd_str)
                .current_dir(cwd)
                .status()
                .map_err(|e| format!("sandbox-exec failed: {}", e))?;

            Ok(status.code().unwrap_or(1))
        }

        SandboxCapability::LinuxLandlock { .. } => {
            // Try bubblewrap first
            let bwrap_check = std::process::Command::new("which")
                .arg("bwrap")
                .output();

            if let Ok(output) = bwrap_check {
                if output.status.success() {
                    // Use bwrap
                    let bwrap_args = linux::generate_bwrap_command(&policy.fence, cwd);
                    let full_cmd = format!("{} {}", bwrap_args, cmd_str);
                    let status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&full_cmd)
                        .current_dir(cwd)
                        .status()
                        .map_err(|e| format!("bwrap failed: {}", e))?;
                    return Ok(status.code().unwrap_or(1));
                }
            }

            Err("bubblewrap (bwrap) not found. Install: apt install bubblewrap".to_string())
        }

        SandboxCapability::None => {
            Err("No sandbox available. Running unsandboxed is not supported.".to_string())
        }
    }
}
