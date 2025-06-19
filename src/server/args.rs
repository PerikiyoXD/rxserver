// SPDX-License-Identifier: Apache-2.0

// RX X11 Server - Command Line Arguments

use crate::plugins::DisplayMode;

#[derive(clap::Parser)]
#[command(about = "RX Server - Rust X11 Compatible Server")]
pub struct ServerArgs {
    #[arg(short, long, default_value = ":0")]
    pub display: String,
    #[arg(short, long, default_value = "rxserver.toml")]
    pub config: DisplayMode,
    #[arg(
        long,
        default_value = "headless",
        help = "Display mode: headless, windowed, or hardware"
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
