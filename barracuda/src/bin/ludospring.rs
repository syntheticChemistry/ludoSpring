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

fn cmd_certify(max_tier: u8) {
    #[cfg(feature = "guidestone")]
    {
        let result = ludospring_barracuda::certification::certify(max_tier);
        std::process::exit(result.exit_code());
    }
    #[cfg(not(feature = "guidestone"))]
    {
        let _ = max_tier;
        eprintln!(
            "[certify] Requires `guidestone` feature. Build with: cargo build --features guidestone"
        );
        std::process::exit(2);
    }
}

fn cmd_validate(
    tier_filter: Option<String>,
    track_filter: Option<String>,
    scenario_id: Option<String>,
    list: bool,
    format_json: bool,
) {
    use ludospring_barracuda::validation::scenarios::{self, Tier};

    let registry = scenarios::build_registry();

    if list {
        if format_json {
            let items: Vec<_> = registry
                .all()
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.meta.id,
                        "tier": s.meta.tier.to_string(),
                        "track": s.meta.track.to_string(),
                        "crate": s.meta.provenance_crate,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string(&items).unwrap_or_default());
        } else {
            println!("Available scenarios ({}):", registry.len());
            for s in registry.all() {
                println!(
                    "  {} [{}] (tier: {}, track: {})",
                    s.meta.id, s.meta.provenance_crate, s.meta.tier, s.meta.track
                );
            }
        }
        return;
    }

    let tier = tier_filter.as_deref().and_then(Tier::from_str_loose);
    let scenarios: Vec<_> = if let Some(id) = &scenario_id {
        registry.find(id).into_iter().collect()
    } else if let Some(t) = tier {
        registry.filter_by_tier(t)
    } else {
        registry.filter_by_tier(Tier::Rust)
    };

    let _ = track_filter;

    if !format_json {
        println!("Running {} scenario(s)...\n", scenarios.len());
    }
    let mut total_pass = 0_u32;
    let mut total_fail = 0_u32;
    let mut scenario_results: Vec<serde_json::Value> = Vec::new();

    for s in &scenarios {
        if !format_json {
            println!("━━━ {} ━━━", s.meta.id);
            println!("  {}", s.meta.description);
        }
        let mut h = ludospring_barracuda::validation::ValidationHarness::new(s.meta.id);
        (s.run)(&mut h);
        let (p, f) = h.counts();
        total_pass += p;
        total_fail += f;

        if format_json {
            scenario_results.push(serde_json::json!({
                "id": s.meta.id,
                "passed": p,
                "failed": f,
                "status": if f == 0 { "PASS" } else { "FAIL" },
            }));
        } else {
            println!();
        }
    }

    if format_json {
        let output = serde_json::json!({
            "status": if total_fail == 0 { "PASS" } else { "FAIL" },
            "checks": total_pass + total_fail,
            "passed": total_pass,
            "failed": total_fail,
            "scenarios": scenario_results,
        });
        println!("{}", serde_json::to_string(&output).unwrap_or_default());
    } else {
        println!("═══ Summary: {total_pass} passed, {total_fail} failed ═══");
    }
    if total_fail > 0 {
        std::process::exit(1);
    }
}

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

    server
        .run_until(shutdown.as_ref())
        .map_err(classify_io_error)?;

    ludospring_barracuda::biomeos::deregister_domain();
    info!("Shutdown complete");
    Ok(())
}

fn cmd_status() {
    use ludospring_barracuda::ipc::methods;

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
    match ludospring_barracuda::ipc::call_primal(&endpoint, methods::health::LIVENESS, &params)
        .or_else(|_| {
            ludospring_barracuda::ipc::call_primal(&endpoint, methods::lifecycle::STATUS, &params)
        })
        .or_else(|_| {
            ludospring_barracuda::ipc::call_primal(&endpoint, methods::health::CHECK, &params)
        }) {
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
    /// Run certification (guidestone three-tier validation).
    Certify {
        /// Maximum tier to validate (1=bare, 2=IPC, 3=NUCLEUS).
        #[arg(long, default_value = "3")]
        tier: u8,
    },
    /// Run validation scenarios (absorbed experiments).
    Validate {
        /// Filter by tier: rust, live, both/all.
        #[arg(long)]
        tier: Option<String>,
        /// Filter by track: interaction, procedural, engagement, composition, performance.
        #[arg(long)]
        track: Option<String>,
        /// Run a specific scenario by id.
        #[arg(long)]
        scenario: Option<String>,
        /// List available scenarios without running them.
        #[arg(long)]
        list: bool,
        /// Output format: omit for human-readable, "json" for structured output (Tier 2).
        #[arg(long)]
        format: Option<String>,
    },
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
    let result: Result<(), commands::CliError> = match cli.command {
        Command::Certify { tier } => {
            cmd_certify(tier);
            Ok(())
        }
        Command::Validate {
            tier,
            track,
            scenario,
            list,
            format,
        } => {
            cmd_validate(
                tier,
                track,
                scenario,
                list,
                format.as_deref() == Some("json"),
            );
            Ok(())
        }
        Command::Server { port } => cmd_server(port).map_err(Into::into),
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
