use clap::{Parser, Subcommand};

/// Root of all command line arguments
#[derive(Debug, Parser)]
#[command(name = "p2p-puncher")]
#[command(about = "Connect two computers behind NAT together with UDP hole punching.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Work a server proxying incoming clients to a destination
    #[command(arg_required_else_help = true)]
    Server {
        /// Where should data be forwarded
        forward: String,
        /// The address of TURN server
        turn: String,
        /// The name of current service
        service: String,
    },
    /// Work as a client connecting to remote server
    #[command(arg_required_else_help = true)]
    Client {
        /// Listen on this address
        listen: String,
        /// The address of TURN server
        turn: String,
        /// The name of current service
        service: String,
    },
    /// Work as TURN server
    #[command(arg_required_else_help = true)]
    TURN {
        /// Listen on this address
        listen: String,
    },
}
