use crate::{DbControl, Frame};

#[derive(Debug)]
pub enum CommandType {
    Set,
    Get,
    Del,
    Ping,
}

#[derive(Debug)]
pub struct Command {
    cmd: CommandType,
    key: Option<String>,
    value: Option<String>,
}

impl Command {
    pub fn new_set<S: ToString>(k: S, v: S) -> Self {
        Self {
            cmd: CommandType::Set,
            key: Some(k.to_string()),
            value: Some(v.to_string()),
        }
    }

    pub fn new_get<S: ToString>(k: S) -> Self {
        Self {
            cmd: CommandType::Get,
            key: Some(k.to_string()),
            value: None,
        }
    }

    pub fn new_del<S: ToString>(k: S) -> Self {
        Self {
            cmd: CommandType::Del,
            key: Some(k.to_string()),
            value: None,
        }
    }

    pub fn new_ping() -> Self {
        Self {
            cmd: CommandType::Ping,
            key: None,
            value: None,
        }
    }

    pub fn apply(self, db: &DbControl) -> Result<Option<String>, String> {
        let key = self.key.ok_or(format!("missing key"));
        let value = self.value.ok_or(format!("missing value"));

        let res = match self.cmd {
            CommandType::Set => db.set(key?, value?),
            CommandType::Get => db.get(&key?),
            CommandType::Del => db.del(&key?),
            CommandType::Ping => Some(format!("PONG")),
        };

        Ok(res)
    }

    pub fn into_frame(self) -> Result<Frame, String> {
        let mut frame = Frame::new(&self.cmd);
        let key = self.key.ok_or(format!("missing key"));
        let value = self.value.ok_or(format!("missing value"));

        match self.cmd {
            CommandType::Set => {
                frame.push_arg(key?);
                frame.push_arg(value?);
            }
            CommandType::Get => frame.push_arg(key?),
            CommandType::Del => frame.push_arg(key?),
            CommandType::Ping => {}
        };

        Ok(frame)
    }

    pub fn from_frame(frame: &Frame) -> Result<Self, String> {
        match &frame {
            Frame::Array(arr) => parse_cmd(arr),
            Frame::Cmd(ping) => {
                let cmd = std::str::from_utf8(ping).map_err(|e| format!("{}", e))?;
                if cmd == "ping" {
                    return Ok(Command::new_ping());
                } else {
                    return Err(format!("error frame"));
                }
            }
            _ => Err(format!("error frame")),
        }
    }
}

fn parse_cmd(arr: &Vec<Frame>) -> Result<Command, String> {
    let Frame::Cmd(cmd) = parse_arr_index(arr, 0)? else{
        return Err(format!("not a Frame::Cmd"));
    };

    let k = parse_value(parse_arr_index(arr, 1)?);

    match std::str::from_utf8(cmd).map_err(|e| format!("{}", e))? {
        "set" => Ok(Command::new_set(k?, parse_value(parse_arr_index(arr, 2)?)?)),
        "get" => Ok(Command::new_get(k?)),
        "del" => Ok(Command::new_del(k?)),
        _ => Err(format!("not a Frame::Cmd")),
    }
}

fn parse_value(frame: &Frame) -> Result<String, String> {
    match frame {
        Frame::Cmd(val) | Frame::Arg(val) => {
            String::from_utf8(val.to_vec()).map_err(|e| format!("{}", e))
        }
        _ => Err(format!("error frame")),
    }
}

fn parse_arr_index(arr: &Vec<Frame>, index: usize) -> Result<&Frame, String> {
    arr.get(index)
        .ok_or(format!("out of index from Frame::Array"))
}
