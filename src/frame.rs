use crate::command::CommandType;
use bytes::{Buf, Bytes};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub enum Frame {
    Cmd(Bytes),        // +
    Arg(Bytes),        // -
    Error(Bytes),      // ?
    Array(Vec<Frame>), // *
    Null,              // !
}

impl Frame {
    pub fn new(cmd: &CommandType) -> Self {
        if let CommandType::Ping = cmd {
            return Self::Cmd(Bytes::from_static(b"ping"));
        }
        Self::new_arr(cmd)
    }

    pub fn new_arg(arg: String) -> Self {
        Frame::Arg(Bytes::from(arg))
    }

    pub fn new_err(err: String) -> Self {
        Frame::Arg(Bytes::from(err))
    }

    pub fn push_arg(&mut self, arg: String) {
        if let Self::Array(arr) = self {
            arr.push(Self::Arg(Bytes::from(arg)));
        }
    }

    fn new_arr(cmd: &CommandType) -> Self {
        let mut arr = Vec::with_capacity(3);
        let bytes = match cmd {
            CommandType::Set => Bytes::from_static(b"set"),
            CommandType::Get => Bytes::from_static(b"get"),
            CommandType::Del => Bytes::from_static(b"del"),
            CommandType::Ping => Bytes::from_static(b"ping"),
        };

        arr.push(Frame::Cmd(bytes));

        Frame::Array(arr)
    }
    pub fn len(&self) -> usize {
        match self {
            Frame::Cmd(val) | Frame::Arg(val) | Frame::Error(val) => val.len(),
            Frame::Array(arr) => arr.len(),
            Frame::Null => 0,
        }
    }
}

fn get_u8(cur: &mut Cursor<&[u8]>) -> Option<u8> {
    if !cur.has_remaining() {
        return None;
    }
    Some(cur.get_u8())
}

fn parse_line<'a, 'b>(cur: &'a mut Cursor<&'b [u8]>) -> Option<&'a [u8]> {
    let now = cur.position() as usize;
    let len = cur.get_ref().len();

    for i in now..len {
        if cur.get_ref()[i] == b'\r' && cur.get_ref()[i + 1] == b'\n' {
            cur.set_position((i + 2) as u64);
            return Some(&cur.get_ref()[now..i]);
        }
    }
    None
}

fn parse_num(src: &[u8]) -> Option<usize> {
    let len = std::str::from_utf8(src).ok()?;
    len.parse::<usize>().ok()
}

fn check_val(cur: &mut Cursor<&[u8]>) -> Option<Bytes> {
    let len = parse_num(parse_line(cur)?)?;
    let data = parse_line(cur)?;
    if data.len() != len {
        return None;
    }

    Some(Bytes::copy_from_slice(data))
}

pub fn parse_frame(cur: &mut Cursor<&[u8]>) -> Option<Frame> {
    let frame = match get_u8(cur)? {
        b'+' => Frame::Cmd(check_val(cur)?),
        b'-' => Frame::Arg(check_val(cur)?),
        b'?' => Frame::Error(check_val(cur)?),
        b'*' => {
            let num = parse_num(parse_line(cur)?)?;
            let mut arr = Vec::with_capacity(3);
            for _ in 0..num {
                arr.push(parse_frame(cur)?)
            }
            Frame::Array(arr)
        }
        _ => Frame::Null,
    };

    Some(frame)
}
