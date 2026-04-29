//! OpenTUI integration for pawlos (Unix-only)
//! Provides enhanced terminal UI using opentui_rust

#[cfg(unix)]
pub async fn run_opentui_welcome(agent_name: &str, gateway_url: Option<&str>) {
    use opentui_rust::{
        Terminal, Renderer, OptimizedBuffer,
        style::{Style, Rgba},
        ansi::AnsiCodec,
    };
    use std::time::Duration;
    
    // Enter raw mode
    let _raw_guard = match Terminal::enter_raw_mode() {
        Ok(guard) => guard,
        Err(_) => {
            eprintln!("Failed to enter raw mode, falling back to simple UI");
            return;
        }
    };
    
    // Get terminal size
    let (cols, rows) = match Terminal::size() {
        Ok((c, r)) => (c as usize, r as usize),
        Err(_) => (80, 24),
    };
    
    // Create buffers
    let mut front = OptimizedBuffer::new(cols, rows);
    let mut back = OptimizedBuffer::new(cols, rows);
    
    // Clear screen
    front.clear();
    
    // Draw welcome message
    let welcome = format!("🤖 pawlos — {} ready", agent_name);
    front.set_str(2, 2, &welcome, Style::new().fg(Rgba::new(0, 255, 255, 255))); // Cyan
    
    if let Some(url) = gateway_url {
        let gateway = format!("🌐 Gateway: {}", url);
        front.set_str(2, 4, &gateway, Style::new().fg(Rgba::new(255, 0, 255, 255))); // Magenta
    }
    
    front.set_str(2, 6, "Press any key to start...", Style::new().fg(Rgba::new(128, 128, 128, 255)));
    
    // Render
    let mut renderer = Renderer::new(front.width(), front.height());
    if let Ok(mut stdout) = std::io::stdout().lock() {
        let _ = renderer.render_diff(&front, &back, &mut stdout);
        let _ = stdout.flush();
    }
    
    // Wait for key press
    let mut buf = [0u8; 1];
    let _ = std::io::stdin().read_exact(&mut buf);
    
    // Clear screen on exit
    print!("\x1b[2J\x1b[H");
}

#[cfg(not(unix))]
pub async fn run_opentui_welcome(_agent_name: &str, _gateway_url: Option<&str>) {
    // No-op on non-Unix systems
}
