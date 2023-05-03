use crate::{Command, Connection, Frame};
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    connection: Connection,
}

pub async fn connect(addr: SocketAddr) -> std::io::Result<Client> {
    let stream = TcpStream::connect(&addr).await?;

    Ok(Client {
        connection: Connection::new(stream),
    })
}

impl Client {
    pub async fn set<S: ToString>(&mut self, key: S, value: S) -> Result<String, String> {
        let req_frame = Command::new_set(key, value).into_frame()?;
        execute(&mut self.connection, req_frame).await
    }

    pub async fn get<S: ToString>(&mut self, key: S) -> Result<String, String> {
        let req_frame = Command::new_get(key).into_frame()?;
        execute(&mut self.connection, req_frame).await
    }

    pub async fn del<S: ToString>(&mut self, key: S) -> Result<String, String> {
        let req_frame = Command::new_del(key).into_frame()?;
        execute(&mut self.connection, req_frame).await
    }

    pub async fn ping(&mut self) -> Result<String, String> {
        let req_frame = Command::new_ping().into_frame()?;
        execute(&mut self.connection, req_frame).await
    }
}

async fn execute(connection: &mut Connection, req_frame: Frame) -> Result<String, String> {
    match connection.write_frame(&req_frame).await {
        Ok(()) => {
            let resp_frame = connection
                .read_frame()
                .await
                .ok_or(format!("read frame failed"))?;

            if let Frame::Arg(arg) = resp_frame {
                return String::from_utf8(arg.to_vec()).map_err(|e| format!("{}", e));
            } else if let Frame::Null = resp_frame {
                return Ok(format!(""));
            } else {
                return Err(format!("error frame"));
            }
        }
        Err(e) => Err(format!("{}", e)),
    }
}
