use clap::Parser;

mod arguments;
mod client;
mod messages;
mod server;
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
        } => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                server::spawn_server(&forward, &stun, &service).await;
            }),
        arguments::Commands::Client {
            listen,
            stun,
            service,
        } => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                client::spawn_client(&listen, &stun, &service).await;
            }),
        arguments::Commands::STUN { listen } => {
            stun::spawn_stun(&listen);
        }
    };
}
