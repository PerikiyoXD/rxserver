// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

// RX X11 Server - Command Line Arguments

#[derive(clap::Parser)]
#[command(about = "RX Server - Rust X11 Compatible Server")]
pub struct CommandlineArgs {
    #[arg(short, long, default_value = ":0")]
    pub display: String,
    #[arg(short, long, default_value = "rxserver.toml")]
    pub config: String,
    #[arg(
        long,
        default_value = "headless",
        help = "Display mode: headless, virtual, native"
    )]
    pub mode: String,
    #[arg(
        long,
        default_value = "1920",
        help = "Display width (for virtual/native mode)"
    )]
    pub width: u32,
    #[arg(
        long,
        default_value = "1080",
        help = "Display height (for virtual/native mode)"
    )]
    pub height: u32,
}
