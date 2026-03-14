// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring UniBin — game science primal for biomeOS deployment.
//!
//! Per Spring-as-Niche Deployment Standard: single binary with `server`, `status`, `version`.
//! Socket: `$XDG_RUNTIME_DIR/biomeos/ludospring-${FAMILY_ID}.sock` (overridable via env).
#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown, clippy::module_name_repetitions)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand};
use ludospring_barracuda::PRIMAL_NAME;
use ludospring_barracuda::ipc::{
    IpcServer, METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_BEGIN_SESSION,
    METHOD_COMPLETE_SESSION, METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW,
    METHOD_FITTS_COST, METHOD_GENERATE_NOISE, METHOD_RECORD_ACTION, METHOD_WFC_STEP,
};
use tracing::info;

/// Capabilities exposed by ludoSpring (capability-based, no hardcoded primal names).
const CAPABILITIES: &[&str] = &[
    METHOD_EVALUATE_FLOW,
    METHOD_FITTS_COST,
    METHOD_ENGAGEMENT,
    METHOD_ANALYZE_UI,
    METHOD_ACCESSIBILITY,
    METHOD_WFC_STEP,
    METHOD_DIFFICULTY_ADJUSTMENT,
    METHOD_GENERATE_NOISE,
    METHOD_BEGIN_SESSION,
    METHOD_RECORD_ACTION,
    METHOD_COMPLETE_SESSION,
];

/// Ecosystem socket directory name — XDG convention for biomeOS primals.
const BIOMEOS_DIR: &str = "biomeos";

/// Socket prefix for Neural API — discovered at runtime via lifecycle.register.
/// Peer primal names are not hardcoded; this is the conventional socket name
/// for the Neural API capability in the biomeOS ecosystem.
const NEURAL_API_SOCKET_PREFIX: &str = "neural-api";

fn get_family_id() -> String {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Resolve socket path using XDG-compliant priority:
///
/// 1. `LUDOSPRING_SOCK` / `LUDOSPRING_SOCKET` — explicit override
/// 2. `BIOMEOS_SOCKET_DIR` + `/ludospring-{family_id}.sock`
/// 3. `$XDG_RUNTIME_DIR/biomeos/ludospring-{family_id}.sock`
/// 4. `/tmp/biomeos-$USER/ludospring-{family_id}.sock`
/// 5. `/tmp/ludospring-{family_id}.sock`
fn resolve_socket_path(family_id: &str) -> PathBuf {
    if let Ok(explicit) = std::env::var("LUDOSPRING_SOCK") {
        return PathBuf::from(explicit);
    }
    if let Ok(explicit) = std::env::var("LUDOSPRING_SOCKET") {
        return PathBuf::from(explicit);
    }

    let sock_name = format!("{PRIMAL_NAME}-{family_id}.sock");

    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(&sock_name);
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let biomeos_path = PathBuf::from(xdg).join(BIOMEOS_DIR);
        if biomeos_path.is_dir() || std::fs::create_dir_all(&biomeos_path).is_ok() {
            return biomeos_path.join(&sock_name);
        }
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let fallback = PathBuf::from("/tmp").join(format!("biomeos-{user}"));
    if fallback.is_dir() || std::fs::create_dir_all(&fallback).is_ok() {
        return fallback.join(&sock_name);
    }

    PathBuf::from("/tmp").join(&sock_name)
}

/// Discover Neural API socket path (same runtime dir, discovered by capability).
fn neural_api_socket_path(family_id: &str) -> PathBuf {
    let sock_name = format!("{NEURAL_API_SOCKET_PREFIX}-{family_id}.sock");
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(&sock_name);
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg).join(BIOMEOS_DIR).join(&sock_name);
    }
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    PathBuf::from("/tmp")
        .join(format!("biomeos-{user}"))
        .join(&sock_name)
}

/// Send a JSON-RPC request to a Unix socket and return the raw response.
fn json_rpc_call(
    socket_path: &std::path::Path,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let request_str = serde_json::to_string(&request).ok()?;
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.write_all(request_str.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;
    stream.flush().ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;

    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    let line = buf.lines().next()?;
    serde_json::from_str(line).ok()
}

/// Register with biomeOS Neural API if available.
fn register_with_neural_api(socket_path: &std::path::Path, family_id: &str) {
    let neural_path = neural_api_socket_path(family_id);
    if !neural_path.exists() {
        info!(
            "Neural API not found at {} — running standalone",
            neural_path.display()
        );
        return;
    }

    let params = serde_json::json!({
        "name": PRIMAL_NAME,
        "socket_path": socket_path.to_string_lossy(),
        "pid": std::process::id(),
        "capabilities": CAPABILITIES,
    });

    if json_rpc_call(&neural_path, "lifecycle.register", &params).is_some() {
        info!(
            "Registered with Neural API: {} capabilities",
            CAPABILITIES.len()
        );
    } else {
        tracing::warn!("lifecycle.register failed (non-fatal) — Neural API unreachable");
    }
}

fn cmd_server() -> Result<(), String> {
    let family_id = get_family_id();
    let socket_path = resolve_socket_path(&family_id);

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create socket dir {}: {e}", parent.display()))?;
    }

    let server = IpcServer::with_path(&socket_path);

    info!("ludospring IPC listening on {}", socket_path.display());
    info!("  Family ID: {family_id}");
    info!("  Version: {}", env!("CARGO_PKG_VERSION"));
    info!("  Capabilities ({}):", CAPABILITIES.len());
    for cap in CAPABILITIES {
        info!("    - {cap}");
    }

    register_with_neural_api(&socket_path, &family_id);

    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))
        .map_err(|e| format!("Failed to install SIGTERM handler: {e}"))?;

    server
        .run_until(shutdown.as_ref())
        .map_err(|e| format!("Server error: {e}"))?;
    info!("Shutdown complete");
    Ok(())
}

fn cmd_status() {
    let family_id = get_family_id();
    let socket_path = resolve_socket_path(&family_id);

    if !socket_path.exists() {
        eprintln!(
            "ludospring not running (socket not found: {})",
            socket_path.display()
        );
        eprintln!("  Family ID: {family_id}");
        eprintln!("  Start with: ludospring server");
        return;
    }

    let params = serde_json::json!({});
    match json_rpc_call(&socket_path, "health.check", &params)
        .or_else(|| json_rpc_call(&socket_path, "lifecycle.health", &params))
    {
        Some(resp) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp).unwrap_or_else(|_| resp.to_string())
            );
        }
        None => {
            eprintln!("Socket exists but health check failed — server may be busy or unresponsive");
        }
    }
}

fn cmd_version() {
    println!("ludospring {}", env!("CARGO_PKG_VERSION"));
    println!("  Primal: {PRIMAL_NAME}");
    println!("  Capabilities: {}", CAPABILITIES.join(", "));
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
