use crate::{Command, Connection, DbControl, Frame};
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};

pub async fn run(addr: SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    let shutdonw = signal::ctrl_c();

    let mut server = Server {
        listener,
        db: DbControl::new(),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                tracing::error!("{}",err);
            }
        }
        _ = shutdonw => {
            if let Err(err) = server.db.backup() {
                tracing::error!("{}",err);
            } else {
                tracing::info!("data was backup");
            }
        },
    }

    Ok(())
}

struct Server {
    listener: TcpListener,
    db: DbControl,
}

impl Server {
    pub async fn get_connection(&self) -> std::io::Result<Connection> {
        let (stream, _) = self.listener.accept().await.unwrap();
        Ok(Connection::new(stream))
    }

    async fn run(&mut self) -> std::io::Result<()> {
        loop {
            let connection = self.get_connection().await?;
            let db = self.db.clone();

            tokio::spawn(async move {
                if let Err(err) = execute(connection, &db).await {
                    tracing::error!(err);
                }
            });
        }
    }
}

async fn execute(mut connection: Connection, db: &DbControl) -> Result<(), String> {
    let req_frame = connection
        .read_frame()
        .await
        .ok_or(format!("read frame failed"))?;

    tracing::info!(?req_frame);

    let res = Command::from_frame(&req_frame)?.apply(db)?;

    let resp_frame = match res {
        Some(s) => Frame::new_arg(s),
        None => Frame::Null,
    };

    connection
        .write_frame(&resp_frame)
        .await
        .map_err(|e| format!("{}", e))
}
