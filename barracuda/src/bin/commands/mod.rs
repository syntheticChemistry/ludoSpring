// SPDX-License-Identifier: AGPL-3.0-or-later

mod dashboard;
mod hud_fixtures;
mod live_session;
mod tufte_dashboard;

pub use dashboard::cmd_dashboard;
pub use live_session::cmd_live_session;
pub use tufte_dashboard::cmd_tufte_dashboard;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("serialize: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error(transparent)]
    Ipc(#[from] ludospring_barracuda::ipc::IpcError),
}

impl CliError {
    pub fn io(context: impl std::fmt::Display, source: std::io::Error) -> Self {
        Self::Io {
            context: context.to_string(),
            source,
        }
    }
}
