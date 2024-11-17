use bytes::{BufMut, BytesMut};
use crate::message::Write;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Numeric {
    /// Reply 315
    RPL_ENDOFWHO{client: String, mask: String},
    /// Reply 352
    RPL_WHOREPLY{client: String, channel: String, username: String, host: String, server: String, nick: String, flags: String, hopcount: String, realname: String},
    /// Reply 353
    RPL_NAMREPLY{client: String, symbol: String, channel: String, members: Vec<String>},
    /// Reply 366
    RPL_ENDOFNAMES{client: String, channel: String},
    /// Reply 372
    RPL_MOTD{client: String, line: String},
    /// Reply 375
    RPL_MOTDSTART{client: String, line: String},
    /// Reply 376
    RPL_ENDOFMOTD{client: String},

    /// Error 412
    ERR_NOTEXTTOSEND{client: String},
    /// Error 431
    ERR_NONICKNAMEGIVEN{client: String},
    /// Error 432
    ERR_ERRONEUSNICKNAME{client: String, nick: String},
    /// Error 433
    ERR_NICKNAMEINUSE{client: String, nick: String},
    /// Error 436
    ERR_NICKCOLLISION{client: String, nick: String, user: String, host: String},
    /// Error 461
    ERR_NEEDMOREPARAMS{client: String, command: String},
    /// Error 462
    ERR_ALREADYREGISTERED{client: String},
    /// Error 464
    ERR_PASSWDMISMATCH{client: String}, // 464

    /// Custom
    UNKNOWN{numeric: String, params: Vec<String>}
}

impl Numeric {
    pub fn new(numeric: &str, params: Vec<String>) -> Self {
        use Numeric::*;

        let mut params_iter = params.into_iter();

        macro_rules! required {
            () => {
                match params_iter.next() {
                    Some(param) => param,
                    None => return UNKNOWN{numeric: numeric.to_string(), params: params_iter.collect()},
                }
            };
        }

        // macro_rules! optional {
        //     () => {
        //         params_iter.next()
        //     };
        // }

        match numeric {
            "315" => RPL_ENDOFWHO{client: required!(), mask: required!()},
            "352" => RPL_WHOREPLY{client: required!(), channel: required!(), username: required!(), host: required!(), server: required!(), nick: required!(), flags: required!(), hopcount: required!(), realname: required!()},
            "353" => RPL_NAMREPLY{client: required!(), symbol: required!(), channel: required!(), members: params_iter.collect()},
            "366" => RPL_ENDOFNAMES{client: required!(), channel: required!()},
            "372" => RPL_MOTD{client: required!(), line: required!()},
            "375" => RPL_MOTDSTART{client: required!(), line: required!()},
            "376" => RPL_ENDOFMOTD{client: required!()},

            "412" => ERR_NOTEXTTOSEND{client: required!()},
            "431" => ERR_NONICKNAMEGIVEN{client: required!()},
            "432" => ERR_ERRONEUSNICKNAME{client: required!(), nick: required!()},
            "433" => ERR_NICKNAMEINUSE{client: required!(), nick: required!()},
            "436" => ERR_NICKCOLLISION{client: required!(), nick: required!(), user: required!(), host: required!()},
            "461" => ERR_NEEDMOREPARAMS{client: required!(), command: required!()},
            "462" => ERR_ALREADYREGISTERED{client: required!()},
            "464" => ERR_PASSWDMISMATCH{client: required!()},

            _ => UNKNOWN{numeric: numeric.to_string(), params: params_iter.collect()}
        }

    }

    pub fn params(&self) -> Vec<String> {
        use Numeric::*;

        match self {
            RPL_ENDOFWHO{client, mask} => vec![client.to_string(), mask.to_string()],
            // RPL_WHOREPLY{client, channel, username, host, server, nick, flags, hopcount, realname} => vec![client, channel, username, host, server, nick, flags, hopcount, realname],
            // RPL_NAMREPLY{client, symbol, channel, members} => vec![client, symbol, channel, members],
            RPL_ENDOFNAMES{client, channel} => vec![client.to_string(), channel.to_string()],
            RPL_MOTD{client, line} => vec![client.to_string(), line.to_string()],
            RPL_MOTDSTART{client, line} => vec![client.to_string(), line.to_string()],
            RPL_ENDOFMOTD{client} => vec![client.to_string()],

            ERR_NOTEXTTOSEND{client} => vec![client.to_string()],
            ERR_NONICKNAMEGIVEN{client} => vec![client.to_string()],
            ERR_ERRONEUSNICKNAME{client, nick} => vec![client.to_string(), nick.to_string()],
            ERR_NICKNAMEINUSE{client, nick} => vec![client.to_string(), nick.to_string()],
            ERR_NICKCOLLISION{client, nick, user, host} => vec![client.to_string(), nick.to_string(), user.to_string(), host.to_string()],
            ERR_NEEDMOREPARAMS{client, command} => vec![client.to_string(), command.to_string()],
            ERR_ALREADYREGISTERED{client} => vec![client.to_string()],
            ERR_PASSWDMISMATCH{client} => vec![client.to_string()],

            _ => vec![],
        }
    }

    pub fn numeric(&self) -> u16 {
        use Numeric::*;

        match self {
            RPL_ENDOFWHO{..} => 315,
            RPL_WHOREPLY{..} => 352,
            RPL_NAMREPLY{..} => 353,
            RPL_ENDOFNAMES{..} => 366,
            RPL_MOTD{..} => 372,
            RPL_MOTDSTART{..} => 375,
            RPL_ENDOFMOTD{..} => 376,

            ERR_NOTEXTTOSEND{..} => 412,
            ERR_NONICKNAMEGIVEN{..} => 431,
            ERR_ERRONEUSNICKNAME{..} => 432,
            ERR_NICKNAMEINUSE{..} => 433,
            ERR_NICKCOLLISION{..} => 436,
            ERR_NEEDMOREPARAMS{..} => 461,
            ERR_ALREADYREGISTERED{..} => 462,
            ERR_PASSWDMISMATCH{..} => 464,

            _ => 0,
        }
    }
}

impl Write for Numeric {
    fn write(&self, buf: &mut bytes::BytesMut) {
        buf.put_slice(self.numeric().to_string().as_bytes());
        buf.put_slice(b" ");
        buf.put_slice(self.params().join(" ").as_bytes());

    }
}
