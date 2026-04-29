use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

pub struct ShellTool;

impl ShellTool {
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "shell",
            "description": "Execute a shell command on the local system. \
                Use for file operations, package installs, running scripts. \
                Destructive commands require user confirmation.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The shell command to run" },
                    "working_dir": { "type": "string", "description": "Optional working directory" },
                    "timeout_secs": { "type": "integer", "description": "Timeout in seconds (default 30)" }
                },
                "required": ["command"]
            }
        })
    }

    /// Blocked dangerous patterns
    const BLOCKED: &'static [&'static str] = &[
        "rm -rf /",
        "dd if=",
        "mkfs",
        ":(){:|:&};:",
        "~/.ssh",
    ];

    pub async fn execute(args: &serde_json::Value) -> Result<String> {
        let command = args["command"].as_str().unwrap_or("").to_string();
        let working_dir = args["working_dir"].as_str().map(|s| s.to_string());
        let timeout_secs = args["timeout_secs"].as_u64().unwrap_or(30);

        // Safety check
        for blocked in Self::BLOCKED {
            if command.contains(blocked) {
                return Ok(format!("⛔ Blocked: command contains restricted pattern '{blocked}'"));
            }
        }

        let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
        let flag  = if cfg!(target_os = "windows") { "/C" } else { "-c" };

        let mut cmd = Command::new(shell);
        cmd.arg(flag).arg(&command)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            cmd.output(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Command timed out after {timeout_secs}s"))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let mut result = format!("exit_code: {exit_code}\n");
        if !stdout.is_empty() { result.push_str(&format!("stdout:\n{stdout}\n")); }
        if !stderr.is_empty() { result.push_str(&format!("stderr:\n{stderr}\n")); }
        Ok(result)
    }
}
