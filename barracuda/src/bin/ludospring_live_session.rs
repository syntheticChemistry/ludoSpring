// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Deprecated standalone entry point — prefer `ludospring live-session`.

#[path = "commands/live_session.rs"]
mod live_session;

fn main() {
    eprintln!(
        "Note: the `ludospring_live_session` binary is deprecated; use `ludospring live-session` instead."
    );

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(e) = live_session::cmd_live_session() {
        eprintln!("[fatal] {e}");
        std::process::exit(1);
    }
}
