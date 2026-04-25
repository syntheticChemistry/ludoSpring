// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring UniBin — game science primal for biomeOS deployment.
//!
//! Per Spring-as-Niche Deployment Standard: single binary with `server`, `status`,
//! `version`, and visualization demo subcommands (`dashboard`, `live-session`,
//! `tufte-dashboard`).
//! Socket: `$XDG_RUNTIME_DIR/biomeos/ludospring-${FAMILY_ID}.sock` (overridable via env).
#![forbid(unsafe_code)]

mod commands;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand};
use ludospring_barracuda::PRIMAL_NAME;
use ludospring_barracuda::biomeos::{GAME_CAPABILITIES, GAME_DOMAIN};
use ludospring_barracuda::ipc::IpcServer;
use ludospring_barracuda::niche;
use tracing::info;

fn cmd_server(port: Option<u16>) -> Result<(), ludospring_barracuda::ipc::IpcError> {
    use ludospring_barracuda::ipc::classify_io_error;

    let family_id = niche::family_id();
    let socket_path = niche::resolve_server_socket();

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).map_err(classify_io_error)?;
    }

    let server = IpcServer::with_path(&socket_path);

    info!("ludospring IPC listening on {}", socket_path.display());
    if let Some(p) = port {
        info!("  Port (genomeBin): {p}");
    }
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
        .map_err(classify_io_error)?;

    server.run_until(shutdown.as_ref()).map_err(classify_io_error)?;

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
    match ludospring_barracuda::ipc::call_primal(&endpoint, "health.liveness", &params)
        .or_else(|_| ludospring_barracuda::ipc::call_primal(&endpoint, "lifecycle.status", &params))
        .or_else(|_| ludospring_barracuda::ipc::call_primal(&endpoint, "health.check", &params))
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
    Server {
        /// TCP port for genomeBin/orchestrator binding (informational — logged only).
        #[arg(long)]
        port: Option<u16>,
    },
    /// Print health and capability info.
    Status,
    /// Print version info.
    Version,
    /// Run game science dashboard (push scenarios to petalTongue).
    Dashboard,
    /// Run live game session streaming demo.
    LiveSession,
    /// Run Tufte validation dashboard.
    TufteDashboard,
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
    let result: Result<(), String> = match cli.command {
        Command::Server { port } => cmd_server(port).map_err(|e| e.to_string()),
        Command::Status => {
            cmd_status();
            Ok(())
        }
        Command::Version => {
            cmd_version();
            Ok(())
        }
        Command::Dashboard => commands::cmd_dashboard(),
        Command::LiveSession => commands::cmd_live_session(),
        Command::TufteDashboard => commands::cmd_tufte_dashboard(),
    };

    if let Err(e) = result {
        eprintln!("[fatal] {e}");
        std::process::exit(1);
    }
}
