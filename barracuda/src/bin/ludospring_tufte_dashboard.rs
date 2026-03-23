// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Deprecated standalone entry point — prefer `ludospring tufte-dashboard`.

#[path = "commands/tufte_dashboard.rs"]
mod tufte_dashboard;

fn main() {
    eprintln!(
        "Note: the `ludospring_tufte_dashboard` binary is deprecated; use `ludospring tufte-dashboard` instead."
    );

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(e) = tufte_dashboard::cmd_tufte_dashboard() {
        eprintln!("[fatal] {e}");
        std::process::exit(1);
    }
}
