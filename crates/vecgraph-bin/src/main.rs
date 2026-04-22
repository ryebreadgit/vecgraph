use clap::Parser;
use kvwrap::{KvStore, LocalConfig, LocalStore, RemoteConfig, RemoteStore};
use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
    sync::Arc,
};
use vecgraph::{VecGraphStore, run_server};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:50051", env = "VECGRAPH_LISTEN_ADDR")]
    listen: SocketAddr,

    #[arg(long, default_value = ".vecgraph_data/", env = "VECGRAPH_DATA_PATH")]
    database: String,

    #[arg(long, default_value = "67108864", env = "VECGRAPH_DATABASE_CACHE_SIZE")] // 64 MiB
    database_cache_size: u64,

    #[arg(long, default_value = "false", env = "VECGRAPH_IS_DATABASE_REMOTE")]
    is_db_remote: bool,

    #[arg(long, default_value = "false", env = "VECGRAPH_VERBOSE")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .init();
    }

    let kv_store: Arc<dyn KvStore> = if args.is_db_remote {
        Arc::new(
            RemoteStore::connect(RemoteConfig {
                endpoint: args.database.to_string(),
                connect_timeout: None,
                request_timeout: None,
                connect_lazy: false,
            })
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("RemoteStore error: {}", e)))?,
        )
    } else {
        Arc::new(
            LocalStore::new(LocalConfig {
                path: args.database.to_string(),
                cache_size: args.database_cache_size,
            })
            .map_err(|e| Error::new(ErrorKind::Other, format!("LocalStore error: {}", e)))?,
        )
    };

    let db = Arc::new(VecGraphStore {
        kv: Box::new(kv_store),
    });

    println!("Starting server on {}", args.listen);
    run_server(db, args.listen)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, format!("Server error: {}", e)))?;

    Ok(())
}
