// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring UniBin — game science primal for biomeOS deployment.
//!
//! Per Spring-as-Niche Deployment Standard: single binary with `server`, `status`, `version`.
//! Socket: `$XDG_RUNTIME_DIR/biomeos/ludospring-${FAMILY_ID}.sock` (overridable via env).
#![forbid(unsafe_code)]

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand};
use ludospring_barracuda::PRIMAL_NAME;
use ludospring_barracuda::biomeos::{GAME_CAPABILITIES, GAME_DOMAIN};
use ludospring_barracuda::ipc::IpcServer;
use ludospring_barracuda::niche;
use tracing::info;

fn cmd_server() -> Result<(), String> {
    let family_id = niche::family_id();
    let socket_path = niche::resolve_server_socket();

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create socket dir {}: {e}", parent.display()))?;
    }

    let server = IpcServer::with_path(&socket_path);

    info!("ludospring IPC listening on {}", socket_path.display());
    info!("  Family ID: {family_id}");
    info!("  Domain: {GAME_DOMAIN}");
    info!("  Version: {}", env!("CARGO_PKG_VERSION"));
    info!("  Capabilities ({}):", GAME_CAPABILITIES.len());
    for cap in GAME_CAPABILITIES {
        info!("    - {cap}");
    }

    ludospring_barracuda::biomeos::register_domain(&socket_path);

    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .map_err(|e| format!("Failed to install SIGTERM handler: {e}"))?;

    server
        .run_until(shutdown.as_ref())
        .map_err(|e| format!("Server error: {e}"))?;

    ludospring_barracuda::biomeos::deregister_domain();
    info!("Shutdown complete");
    Ok(())
}

fn cmd_status() {
    let family_id = niche::family_id();
    let socket_path = niche::resolve_server_socket();

    if !socket_path.exists() {
        eprintln!(
            "ludospring not running (socket not found: {})",
            socket_path.display()
        );
        eprintln!("  Family ID: {family_id}");
        eprintln!("  Start with: ludospring server");
        return;
    }

    let endpoint = ludospring_barracuda::ipc::PrimalEndpoint {
        socket: socket_path,
        name: PRIMAL_NAME.to_owned(),
        capabilities: vec![],
    };

    let params = serde_json::json!({});
    match ludospring_barracuda::ipc::call_primal(&endpoint, "health.check", &params)
        .or_else(|_| ludospring_barracuda::ipc::call_primal(&endpoint, "lifecycle.health", &params))
    {
        Ok(resp) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp).unwrap_or_else(|_| resp.to_string())
            );
        }
        Err(e) => {
            eprintln!("Socket exists but health check failed: {e}");
        }
    }
}

fn cmd_version() {
    println!("ludospring {}", env!("CARGO_PKG_VERSION"));
    println!("  Primal: {PRIMAL_NAME}");
    println!("  Domain: {GAME_DOMAIN}");
    println!("  License: AGPL-3.0-or-later");
    println!("  Capabilities ({}):", GAME_CAPABILITIES.len());
    for cap in GAME_CAPABILITIES {
        println!("    - {cap}");
    }
}

#[derive(Parser)]
#[command(
    name = "ludospring",
    about = "ludoSpring — game science primal for biomeOS"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the IPC server (germination mode).
    Server,
    /// Print health and capability info.
    Status,
    /// Print version info.
    Version,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Server => cmd_server(),
        Command::Status => {
            cmd_status();
            Ok(())
        }
        Command::Version => {
            cmd_version();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("[fatal] {e}");
        std::process::exit(1);
    }
}
