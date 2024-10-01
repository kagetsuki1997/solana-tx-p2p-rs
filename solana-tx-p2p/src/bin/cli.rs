use std::{io, io::Write};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use snafu::ResultExt;
use solana_tx_p2p::web::ApiDoc;
use utoipa::OpenApi;

use crate::{
    command::{NodeCmd, ServerCmd},
    error,
};

#[derive(Debug, Parser)]
#[command(author, long_about = None, version)]
pub struct Cli {
    /// Subcommand with parameters.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print version info and exit
    Version,

    /// Output shell completion code for the specified shell (bash, elvish,
    /// fish, powershell or zsh
    Completion { shell: Shell },

    /// Run server and node
    #[command(visible_alias = "run")]
    Server(Box<ServerCmd>),

    /// Run node only
    Node(Box<NodeCmd>),

    /// Output `OpenApi` document
    OpenApi,
}

impl Cli {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Command::Version => {
                io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .context(error::IoSnafu)?;
            }
            Command::Completion { shell } => {
                let mut command = Self::command();
                let bin_name = command.get_name().to_string();
                clap_complete::generate(shell, &mut command, bin_name, &mut io::stdout());
            }
            Command::Server(cmd) => {
                cmd.run()?;
            }
            Command::Node(cmd) => {
                cmd.run()?;
            }
            Command::OpenApi => {
                io::stdout()
                    .write_all(
                        ApiDoc::openapi()
                            .to_yaml()
                            .expect("ApiDoc should be valid yaml")
                            .as_bytes(),
                    )
                    .context(error::IoSnafu)?;
            }
        }

        Ok(())
    }
}
