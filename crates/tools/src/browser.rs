//! Browser automation tool using browser-use library
//! Provides headless browser control for web automation, scraping, and testing

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BrowserAction {
    /// Navigate to a URL
    Navigate { url: String },
    /// Click an element (by CSS selector or text)
    Click { selector: String },
    /// Fill an input field
    Input { selector: String, text: String },
    /// Get page content/text
    GetContent { selector: Option<String> },
    /// Take a screenshot
    Screenshot { path: Option<String> },
    /// Execute JavaScript
    Evaluate { script: String },
    /// Wait for element to appear
    WaitFor { selector: String, timeout_secs: Option<u64> },
    /// Go back/forward
    GoBack,
    GoForward,
    /// Scroll the page
    Scroll { direction: String, amount: Option<i32> },
}

pub struct BrowserTool;

impl BrowserTool {
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "browser",
            "description": "Control a headless browser for web automation, scraping, and testing. \
                Use navigate to open pages, click to interact, input to fill forms, \
                get_content to extract data, screenshot to capture the page.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["navigate", "click", "input", "get_content", "screenshot", "evaluate", "wait_for", "go_back", "go_forward", "scroll"],
                        "description": "Browser action to perform"
                    },
                    "url": { "type": "string", "description": "URL to navigate to (for navigate action)" },
                    "selector": { "type": "string", "description": "CSS selector or text to match" },
                    "text": { "type": "string", "description": "Text to input (for input action)" },
                    "script": { "type": "string", "description": "JavaScript to execute (for evaluate action)" },
                    "path": { "type": "string", "description": "Screenshot save path" },
                    "direction": { "type": "string", "enum": ["up", "down", "left", "right"], "description": "Scroll direction" },
                    "amount": { "type": "integer", "description": "Scroll amount in pixels" },
                    "timeout_secs": { "type": "integer", "description": "Timeout for wait_for action" }
                },
                "required": ["action"]
            }
        })
    }

    /// Parse a BrowserAction from JSON tool call args
    pub fn parse_action(args: &serde_json::Value) -> Result<BrowserAction> {
        let action = args["action"].as_str().unwrap_or("").to_string();
        
        match action.as_str() {
            "navigate" => Ok(BrowserAction::Navigate {
                url: args["url"].as_str().unwrap_or("").to_string(),
            }),
            "click" => Ok(BrowserAction::Click {
                selector: args["selector"].as_str().unwrap_or("").to_string(),
            }),
            "input" => Ok(BrowserAction::Input {
                selector: args["selector"].as_str().unwrap_or("").to_string(),
                text: args["text"].as_str().unwrap_or("").to_string(),
            }),
            "get_content" => Ok(BrowserAction::GetContent {
                selector: args["selector"].as_str().map(String::from),
            }),
            "screenshot" => Ok(BrowserAction::Screenshot {
                path: args["path"].as_str().map(String::from),
            }),
            "evaluate" => Ok(BrowserAction::Evaluate {
                script: args["script"].as_str().unwrap_or("").to_string(),
            }),
            "wait_for" => Ok(BrowserAction::WaitFor {
                selector: args["selector"].as_str().unwrap_or("").to_string(),
                timeout_secs: args["timeout_secs"].as_u64(),
            }),
            "go_back" => Ok(BrowserAction::GoBack),
            "go_forward" => Ok(BrowserAction::GoForward),
            "scroll" => Ok(BrowserAction::Scroll {
                direction: args["direction"].as_str().unwrap_or("down").to_string(),
                amount: args["amount"].as_i64().map(|v| v as i32),
            }),
            other => anyhow::bail!("Unknown browser action: {}", other),
        }
    }

    /// Get the browser-use script path
    fn get_script_path() -> PathBuf {
        let base = std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("browser_agent.py")
    }

    /// Ensure browser-use is installed
    async fn ensure_installed() -> Result<()> {
        let output = Command::new("python")
            .arg("-c")
            .arg("import browser_use")
            .output()
            .await?;

        if !output.status.success() {
            // Install browser-use
            println!("Installing browser-use...");
            let install = Command::new("pip")
                .args(["install", "browser-use", "playwright"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await?;

            if !install.status.success() {
                anyhow::bail!("Failed to install browser-use: {}", String::from_utf8_lossy(&install.stderr));
            }
            
            // Install playwright browsers
            let _ = Command::new("playwright")
                .args(["install", "chromium"])
                .output()
                .await;
        }

        Ok(())
    }

    /// Execute a browser action
    pub async fn execute(action: BrowserAction) -> Result<String> {
        // Ensure browser-use is installed
        if let Err(e) = Self::ensure_installed().await {
            return Ok(format!("Failed to setup browser: {}", e));
        }

        let script = Self::generate_script(action)?;
        
        // Run the browser agent script
        let output = Command::new("python")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Clean up Python traceback for better UX
            let clean_err = stderr.lines()
                .filter(|l| !l.starts_with("Traceback ") && !l.starts_with("  File ") && !l.starts_with("    "))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(format!("Browser error: {}", clean_err.trim()))
        }
    }

    /// Generate Python script for the action
    fn generate_script(action: BrowserAction) -> Result<String> {
        let script = match action {
            BrowserAction::Navigate { url } => format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Navigate to {}')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, url),

            BrowserAction::Click { selector } => format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Click on element matching "{}"')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, selector),

            BrowserAction::Input { selector, text } => format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Input "{}" into element matching "{}"')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, text.replace("\"", "\\\""), selector),

            BrowserAction::GetContent { selector } => {
                let sel = selector.map(|s| format!(" with selector \"{}\"", s)).unwrap_or_default();
                format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Get page content{}')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, sel)
            },

            BrowserAction::Screenshot { path } => {
                let p = path.unwrap_or_else(|| "screenshot.png".to_string());
                format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Take a screenshot and save to {}')
    await agent.run()
    print('Screenshot saved to {}')

asyncio.run(main())
"#, p, p)
            },

            BrowserAction::Evaluate { script } => format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Execute JavaScript: {}')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, script.replace("\"", "\\\"")),

            BrowserAction::WaitFor { selector, timeout_secs } => {
                let to = timeout_secs.unwrap_or(30);
                format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Wait up to {} seconds for element "{}" to appear')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, to, selector)
            },

            BrowserAction::GoBack => r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Go back to previous page')
    await agent.run()
    print('Navigated back')

asyncio.run(main())
"#.to_string(),

            BrowserAction::GoForward => r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Go forward to next page')
    await agent.run()
    print('Navigated forward')

asyncio.run(main())
"#.to_string(),

            BrowserAction::Scroll { direction, amount } => {
                let amt = amount.unwrap_or(500);
                format!(r#"
import asyncio
from browser_use import Agent, Browser

async def main():
    browser = Browser()
    agent = Agent(browser, 'Scroll {} by {} pixels')
    await agent.run()
    print(agent.history[0].result)

asyncio.run(main())
"#, direction, amt)
            },
        };

        Ok(script)
    }
}