use clap::Parser;

mod arguments;
mod client;
mod defer;
mod messages;
mod server;
mod turn;
mod util;

fn main() {
    env_logger::init();
    // Parse arguments
    match arguments::Cli::parse().command {
        arguments::Commands::Server {
            forward,
            turn,
            service,
        } => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                server::spawn_server(&forward, &turn, &service).await;
            }),
        arguments::Commands::Client {
            listen,
            turn,
            service,
        } => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                client::spawn_client(&listen, &turn, &service).await;
            }),
        arguments::Commands::TURN { listen } => {
            turn::spawn_turn(&listen);
        }
    };
}
