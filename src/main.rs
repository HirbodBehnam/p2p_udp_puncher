use clap::Parser;

mod arguments;
mod messages;
mod stun;
mod util;

fn main() {
    env_logger::init();
    // Parse arguments
    match arguments::Cli::parse().command {
        arguments::Commands::Server {
            forward,
            stun,
            service,
        } => todo!(),
        arguments::Commands::Client {
            listen,
            stun,
            service,
        } => todo!(),
        arguments::Commands::STUN { listen } => {
            stun::spawn_stun(&listen);
        }
    };
}
