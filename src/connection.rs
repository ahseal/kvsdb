use crate::{frame::parse_frame, Frame};
use bytes::BytesMut;
use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(128),
        }
    }

    pub async fn read_frame(&mut self) -> Option<Frame> {
        while let Ok(n) = self.stream.read_buf(&mut self.buffer).await {
            if n == 0 {
                return None;
            }

            tracing::debug!(buf = ?self.buffer,"read_frame");

            let mut cur = Cursor::new(&self.buffer[..n]);
            let frame = parse_frame(&mut cur)?;

            return Some(frame);
        }
        None
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        tracing::debug!(?frame, "write_frame");

        match frame {
            Frame::Array(arr) => {
                self.stream.write_u8(b'*').await?;
                writeln_len(&mut self.stream, frame).await?;
                for f in arr {
                    writeln_val(&mut self.stream, f).await?;
                }
            }
            _ => writeln_val(&mut self.stream, frame).await?,
        };

        self.stream.flush().await?;

        Ok(())
    }
}

async fn writeln_val(stream: &mut BufWriter<TcpStream>, frame: &Frame) -> std::io::Result<()> {
    let src = match frame {
        Frame::Cmd(val) => {
            stream.write_u8(b'+').await?;
            writeln_len(stream, frame).await?;
            Some(val)
        }
        Frame::Arg(val) => {
            stream.write_u8(b'-').await?;
            writeln_len(stream, frame).await?;
            Some(val)
        }
        Frame::Error(e) => {
            stream.write_u8(b'?').await?;
            writeln_len(stream, frame).await?;
            Some(e)
        }
        Frame::Null => {
            stream.write_u8(b'!').await?;
            None
        }
        _ => None,
    };

    if let Some(src) = src {
        stream.write_all(src).await?;
    }

    stream.write_all(b"\r\n").await?;

    Ok(())
}

async fn writeln_len(stream: &mut BufWriter<TcpStream>, frame: &Frame) -> std::io::Result<()> {
    let len = frame.len().to_string();
    stream.write_all(len.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;

    Ok(())
}
