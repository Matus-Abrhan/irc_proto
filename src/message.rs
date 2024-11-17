use bytes::{BufMut, BytesMut};

use crate::command::Command;
use crate::numeric::Numeric;

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub content: Content,
}

#[derive(Debug)]
pub enum Content {
    Command(Command),
    Numeric(Numeric),
    Unknown(),
}

pub trait Write {
    fn write(&self, buf: &mut BytesMut) {}
}

impl Message {
    pub fn new(prefix: Option<String>, content: Content) -> Self {
        Message{prefix, content}
    }

    pub fn command(self) -> Option<Command> {
        use Content::*;

        match self.content {
            Command(command) => Some(command.clone()),
            Numeric(_) => None,
            Unknown() => None,
        }
    }

    pub fn numeric(self) -> Option<Numeric> {
        use Content::*;

        match self.content {
            Numeric(numeric) => Some(numeric.clone()),
            Command(_) => None,
            Unknown() => None,
        }
    }
}

impl Content {
    pub fn new(command_numeric: &str, params: Vec<String>) -> Self {
        let mut params = params.to_owned();
        params = match Command::new(command_numeric, params) {
            Command::UNKNOWN{params, ..} => params,
            command => return Content::Command(command),
        };
        match Numeric::new(command_numeric, params) {
            Numeric::UNKNOWN{..} => (),
            numeric => return Content::Numeric(numeric),
        }
        return Content::Unknown();
    }

}

impl Write for Message {
    fn write(&self, buf: &mut BytesMut) {
        if let Some(prefix) = &self.prefix {
            buf.put_slice(b":");
            buf.put_slice(prefix.as_bytes());
            buf.put_slice(b" ");
        }
        self.content.write(buf);
        buf.put_slice(b"\r\n");
    }
}

impl Write for Content {
    fn write(&self, buf: &mut BytesMut) {
        match self {
            Content::Command(command) => command.write(buf),
            Content::Numeric(command) => command.write(buf),
            Content::Unknown() => (),
        }
    }
}
