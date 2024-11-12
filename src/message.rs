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
        }
    }
    
    pub fn numeric(self) -> Option<Numeric> {
        use Content::*;

        match self.content {
            Command(_) => None,
            Numeric(numeric) => Some(numeric.clone()),
        }
    }
}
