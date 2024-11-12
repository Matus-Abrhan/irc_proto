#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Command {
    // Connection Messages
    CAP,
    PASS{password: String},
    NICK{nickname: String},
    USER{user: String, mode: String, unused: String, realname: String},
    PING{token: String},
    PONG{server: Option<String>, token: String},
    OPER{name: String, password: String},
    QUIT{reason: Option<String>},
    ERROR{reason: String},

    // Channel Operations
    JOIN{channels: String, keys: Option<String>},

    // Sending Messages
    PRIVMSG{targets: String, text: String},

    // User-Based Queries
    WHO{mask: String},

    // Custom
    UNKNOWN{command: String, params: Vec<String>}
}

impl Command {
    pub fn new(command: &str, params: Vec<String>) -> Self {
        use Command::*;

        let mut params_iter = params.into_iter();

        macro_rules! required {
            () => {
                match params_iter.next() {
                    Some(param) => param,
                    None => return UNKNOWN{command: command.to_string(), params: params_iter.collect()},
                }
            };
        }

        macro_rules! optional {
            () => {
                params_iter.next()
            };
        }

        match command {
            "CAP" => CAP{},
            "PASS" => PASS{password: required!()},
            "NICK" => NICK{nickname: required!()},
            "USER" => USER{user: required!(), mode: required!(), unused: required!(), realname: required!()},
            "PING" => PING{token: required!()},
            "PONG" => PONG{token: required!(), server: optional!()},
            "OPER" => OPER{name: required!(), password: required!()},
            "QUIT" => QUIT{reason: optional!()},
            "ERROR" => ERROR{reason: required!()},
            "JOIN" => JOIN{channels: required!(), keys: optional!()},
            "PRIVMSG" => PRIVMSG{targets: required!(), text: required!()},
            "WHO" => WHO{mask: required!()},

            _ => UNKNOWN{command: command.to_string(), params: params_iter.collect()},
        }
    }

    pub fn params(self) -> Vec<String> {
        use Command::*;

        match self {
            PING{token} => vec![token],
            PONG{server, token} => {
                if let Some(server) = server {
                    vec![server, token]
                } else {
                    vec![token]
                }
            }
            JOIN{channels, keys} => std::iter::once(channels).chain(keys).collect(),
            PRIVMSG{targets, text} => vec![targets, text],
            PASS{password} => vec![password],
            NICK{nickname} => vec![nickname],
            USER{user, mode, unused, realname} => vec![user, mode, unused, realname],
            WHO{mask} => vec![mask],
            _ => vec![],
        }
    }

    pub fn command(self) -> String {
        use Command::*;

        match self {
            PING{..} => "PING".to_string(),
            PONG{..} => "PONG".to_string(),
            JOIN{..} => "JOIN".to_string(),
            PRIVMSG{..} => "PRIVMSG".to_string(),
            PASS{..} => "PASS".to_string(),
            NICK{..} => "NICK".to_string(),
            USER{..} => "USER".to_string(),
            WHO{..} => "WHO".to_string(),
            _ => "".to_string(),
        }
    }

}
