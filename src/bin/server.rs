use clap::Parser;
use kvs::{server, DEFAULT_PORT};

#[derive(Debug, Parser)]
#[clap(name = "kvs-server", version, about)]
struct Cli {
    /// Port to set
    #[clap(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Enable log
    #[arg(long, short)]
    log: bool,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    if cli.log {
        enable_log();
    }

    let addr = format!("127.0.0.1:{}", cli.port);

    tracing::info!("listening on http://{:?}", addr);

    server::run(addr.parse().unwrap()).await
}

pub fn enable_log() {
    use tracing_subscriber::{
        fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    };

    tracing_subscriber::registry().with(fmt::layer()).init();
}
