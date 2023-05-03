use clap::{Parser, Subcommand};
use kvs::{client, DEFAULT_PORT};

#[derive(Debug, Parser)]
#[clap(name = "kvs-cli", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Hostname to set
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to set
    #[arg(long, short, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Enable log
    #[arg(long, short)]
    log: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Get the value of key
    Get {
        /// Name of key to set
        key: String,
    },
    /// Set key to the string value
    Set {
        /// Name of key to set
        key: String,

        /// Value to set
        value: String,
    },
    /// Remove a given key
    Del {
        /// Name of key to set
        key: String,
    },
    /// TPing server status
    Ping,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();
    if cli.log {
        enable_log();
    }

    let addr = &*format!("{}:{}", cli.host, cli.port);
    let mut client = client::connect(addr.parse().unwrap()).await.unwrap();

    tracing::info!("{:?}", cli.command);
    let res = match cli.command {
        Command::Get { key } => client.get(key).await,
        Command::Set { key, value } => client.set(key, value).await,
        Command::Del { key } => client.del(key).await,
        Command::Ping => client.ping().await,
    };

    match res {
        Ok(value) => {
            tracing::info!(value);
            println!("{}", value)
        }
        Err(e) => tracing::error!("{}", e),
    }

    Ok(())
}

pub fn enable_log() {
    use tracing_subscriber::{
        fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    };

    tracing_subscriber::registry().with(fmt::layer()).init();
}
