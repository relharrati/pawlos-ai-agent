//! Terminal UI - branding, ASCII art, colors, and animations
//! Provides the distinctive pawlos terminal experience

use std::thread;
use std::time::Duration;
use colored::Colorize;

/// Terminal color codes
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    
    // Standard colors
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    
    // Bright versions
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    
    // Background colors
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_MAGENTA: &str = "\x1b[45m";
    pub const BG_CYAN: &str = "\x1b[46m";
    pub const BG_DEFAULT: &str = "\x1b[49m";
    
    /// Pawlos brand colors - cyan/purple gradient theme
    pub mod brand {
        use super::*;
        pub const PRIMARY: &str = CYAN;       // Main brand color (cyan)
        pub const SECONDARY: &str = MAGENTA; // Accent color (purple/magenta)  
        pub const GRADIENT: [&str; 4] = [BRIGHT_CYAN, CYAN, MAGENTA, BRIGHT_MAGENTA];
        
        // Aliases with fun names
        pub const BLUEBERRY: &str = CYAN;
        pub const GRAPE: &str = MAGENTA;
        pub const LAVENDER: &str = BRIGHT_MAGENTA;
        pub const SKY: &str = BRIGHT_CYAN;
        
        pub const SUCCESS: &str = GREEN;
        pub const WARNING: &str = YELLOW;
        pub const ERROR: &str = RED;
        pub const INFO: &str = BLUE;
        pub const MUTED: &str = DIM;
    }
}

/// The Pawlos Logo - cyan/purple gradient ASCII art
pub mod logo {
    use super::colors::*;
    
    pub const LOBSTER: &str = r#"
      __        __
     /  \      /  \
    |  o  o|  |  o  o|   ___   ___   ___
    |  __  |  |  __  |  | _ | | _ | | _ |
    |      |  |      |  ||_|| ||_|| ||_||
    |______|  |______|  |___| |___| |___|

       p a w l - o s   u r   b u d d y
"#;

    pub fn print() {
        // Gradient effect
        let lines = LOBSTER.lines().collect::<Vec<_>>();
        let colors = [BRIGHT_CYAN, CYAN, MAGENTA, BRIGHT_MAGENTA];
        
        for (i, line) in lines.iter().enumerate() {
            let color = colors[i % colors.len()];
            println!("{}{}", color, line);
        }
    }
    
    pub fn print_colored() {
        print();
    }
}

/// "PAWL-OS" banner - cyan/purple gradient
pub mod banner {
    use super::colors::*;
    
    pub const PAWLOS_BANNER: &str = r#"
  _____   __          ___           ____   _____ 
 |  __ \ /\ \        / / |         / __ \ / ____|
 | |__) /  \ \  /\  / /| |  ______| |  | | (___  
 |  ___/ /\ \ \/  \/ / | | |______| |  | |\___ \ 
 | |  / ____ \  /\  /  | |____    | |__| |____) |
 |_| /_/    \_\/  \/   |______|    \____/|_____/ 
"#;

    pub const TAGLINE: &str = "     P A W L - O S   u r   a g e n t   |   b u d d y";

    pub fn print() {
        // Gradient effect from cyan to magenta
        let lines = PAWLOS_BANNER.lines().collect::<Vec<_>>();
        let colors = [BRIGHT_CYAN, CYAN, MAGENTA, BRIGHT_MAGENTA];
        
        for (i, line) in lines.iter().enumerate() {
            let color = colors[i % colors.len()];
            println!("{}{}{}", color, BOLD, line);
        }
        
        // Print tagline with gradient too
        let tagline_colors = [MAGENTA, BRIGHT_MAGENTA, CYAN, BRIGHT_CYAN];
        let tagline_lines = TAGLINE.lines().collect::<Vec<_>>();
        for (i, line) in tagline_lines.iter().enumerate() {
            let color = tagline_colors[i % tagline_colors.len()];
            println!("{}{}", color, line);
        }
    }
}

/// Terminal dashboard ASCII - cyan/purple gradient
pub mod dashboard {
    use super::colors::*;
    use colored::Colorize;
    
    pub const TERMINAL_ART: &str = r#"
  _   _  ____  ____  ____  ___  __    __    
 / \ / \/  __\/  __\/   _ \/  _ \/  \  /  \ 
 | | | ||  \/||  \/||   / | / \ ||  \/||  \/|
 | |_| ||  __/|  __/|   \_| |-|-||    /|    /
 \____/ \_/   \_/   \____/\_/ \_/\_/\_/\_/\_\
"#;

    pub fn print() {
        // Gradient effect
        let lines = TERMINAL_ART.lines().collect::<Vec<_>>();
        let colors = [BRIGHT_CYAN, CYAN, MAGENTA, BRIGHT_MAGENTA];
        
        for (i, line) in lines.iter().enumerate() {
            let color = colors[i % colors.len()];
            println!("{}{}", color, line);
        }
    }
    
    pub fn print_with_label(label: &str) {
        print();
        println!("{}{}[{}]{} {}", DIM, BRIGHT_CYAN, label, RESET, "Active Session".cyan());
    }
}

/// Pawlos welcome screen with all branding
pub fn print_welcome(agent_name: &str) {
    // Clear screen
    print!("\x1b[2J\x1b[H");
    
    // Logo
    print!("{}", colors::CYAN);
    for line in logo::LOBSTER.lines() {
        println!("{}", line);
        thread::sleep(Duration::from_millis(30));
    }
    
    println!("{}", colors::RESET);
    thread::sleep(Duration::from_millis(100));
    
    // Banner - with random tagline
    print!("{}{}", colors::BRIGHT_CYAN, colors::BOLD);
    for line in banner::PAWLOS_BANNER.lines() {
        println!("{}", line);
        thread::sleep(Duration::from_millis(20));
    }
    // Print tagline
    for line in banner::TAGLINE.lines() {
        println!("{}", line);
    }
    
    println!("{}", colors::RESET);
    thread::sleep(Duration::from_millis(200));
    
    // Dashboard
    println!();
    print!("{}", colors::CYAN);
    for line in dashboard::TERMINAL_ART.lines() {
        println!("{}", line);
        thread::sleep(Duration::from_millis(15));
    }
    
    println!("{}", colors::RESET);
    
    // Random startup message
    quotes::print_random();
    
    // Info
    println!();
    println!("{}{}Welcome, {}!{}", colors::DIM, colors::BOLD, agent_name, colors::RESET);
    println!("{}{}Type a message or /help for commands{}",
             colors::DIM, colors::BRIGHT_CYAN, colors::RESET);
    println!();
}

/// Progress spinner animation
pub struct Spinner {
    frames: Vec<&'static str>,
    current: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
        }
    }
    
    pub fn next(&mut self) -> &str {
        let frame = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();
        frame
    }
    
    pub fn print(&mut self, message: &str) {
        print!("\r{}{} {}", self.next(), colors::CYAN, message);
    }
    
    pub fn done(&self, message: &str) {
        println!("\r{}{}✓{} {}", colors::GREEN, colors::BOLD, colors::RESET, message);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Typewriter effect for terminal text
pub fn typewriter(text: &str, ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        thread::sleep(Duration::from_millis(ms));
    }
    println!();
}

/// Blink effect for important text
pub fn blink(text: &str, times: usize) {
    for _ in 0..times {
        print!("\r{}", text);
        thread::sleep(Duration::from_millis(200));
        print!("\r{}", colors::RESET);
        thread::sleep(Duration::from_millis(200));
    }
    println!("{}", text);
}

/// Loading bar animation
pub struct LoadingBar {
    width: usize,
    fill_char: char,
    empty_char: char,
}

impl LoadingBar {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            fill_char: '█',
            empty_char: '░',
        }
    }
    
    pub fn render(&self, progress: f32) -> String {
        let filled = (progress * self.width as f32) as usize;
        let empty = self.width - filled;
        
        let fill = format!("{}{}", colors::BRIGHT_GREEN, self.fill_char.to_string().repeat(filled));
        let empty_str = format!("{}{}", colors::DIM, self.empty_char.to_string().repeat(empty));
        
        format!("[{}{}{}]", fill, empty_str, colors::RESET)
    }
}

impl Default for LoadingBar {
    fn default() -> Self {
        Self::new(30)
    }
}

/// Clear current line
pub fn clear_line() {
    print!("\r\x1b[2K");
}

/// Terminal prompt with brand styling
pub fn prompt(agent_name: &str) -> String {
    format!("{}{}{}> {}",
             colors::CYAN, colors::BOLD, agent_name, colors::RESET)
}

/// Success message with green check
pub fn success(msg: &str) {
    println!("{}✓{} {}", colors::GREEN, colors::BOLD, msg);
}

/// Error message with red X
pub fn error(msg: &str) {
    println!("{}✗{} {}", colors::RED, colors::BOLD, msg);
}

/// Random funny startup messages
pub mod quotes {
    use super::colors::*;
    
    const QUOTES: &[&str] = &[
        "Loading pixel dreams...",
        "Entropy increasing...",
        "Consulting the digital oracle...",
        "Waking up the neural net...",
        "Compiling sassiness...",
        "Initializing coffee detector...",
        "Reticulating splines...",
        "Feeding the AI hamsters...",
        "Calibrating magic...",
        "Asking the rubber duck...",
        "Simulating productivity...",
        "Counting bits...",
        "Spinning up the hamster wheel...",
        "Downloading more RAM...",
        "Locating administrator...",
        "Aligning the planets...",
        "Calculating infinity...",
        "Breaking things to see how they work...",
        "Optimizing for fun...",
        "Generating response...",
    ];
    
    /// Get a random quote based on system time
    pub fn random() -> &'static str {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        QUOTES[seed % QUOTES.len()]
    }
    
    /// Print random quote with styling
    pub fn print_random() {
        println!("{}{}{}{}",
            DIM,
            "  » ",
            CYAN,
            random()
        );
    }
    
    /// Print random funny tagline for the banner
    pub fn tagline() -> &'static str {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        
        const TAGLINES: &[&str] = &[
            "ur buddy",
            "Ready to assist",
            "It compiles! Ship it!",
            "Powered by coffee and ambition",
            "AI at your service",
            "Now with 20% more intelligence",
            "Click clack goes the code",
            "Don't look at the logs",
            "sudo make me a sandwich",
            "rm -rf /problems",
            "404: Boredom not found",
            "I for one welcome our new robot overlords",
            "It's not a bug, it's a feature",
            "Works on my machine",
            "git push --force",
            "sudo rm -rf /tmp",
            "Bracket balanced, ship it!",
            "Cargo cult engineering",
            "Release early, release often",
            "Your AI bestie",
        ];
        
        TAGLINES[seed % TAGLINES.len()]
    }
}

/// Info message with cyan i
pub fn info(msg: &str) {
    println!("{}ℹ{} {}", colors::CYAN, colors::BOLD, msg);
}

/// Warning message with yellow !
pub fn warn(msg: &str) {
    println!("{}⚠{} {}", colors::YELLOW, colors::BOLD, msg);
}

/// Fun suffixes for agent responses
pub mod responses {
    use super::colors::*;
    
    const THOUGHTS: &[&str] = &[
        "*adjusts virtual collar*",
        "*scurries across the keyboard*",
        "*beeps confidently*",
        "*optimistic beeping intensifies*",
        "*consults the great algorithm*",
        "*nods in binary*",
        "*processes your words carefully*",
        "*examines the prompt from multiple angles*",
        "*calculates optimal response vector*",
        "*rubs digital paws together*",
        "*consults internal documentation*",
        "*runs a quick sanity check*",
        "*cross-references with itself*",
        "*accesses the knowledge base*",
        "*performs a thought operation*",
        "*generates response with 99% confidence*",
        "*compiles an answer*",
        "*checks for gotchas*",
        "*searches through the archives*",
        "*queries the neural pathways*",
    ];
    
    /// Get a random thought prefix
    pub fn random() -> &'static str {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        THOUGHTS[seed % THOUGHTS.len()]
    }
    
    /// Print agent response with fun prefix
    pub fn prefix() -> String {
        format!("{}{}{}", DIM, CYAN, random())
    }
}
