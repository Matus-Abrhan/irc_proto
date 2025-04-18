use std::net::SocketAddr;
use tokio::{self, io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use std::io::Cursor;

use bytes::{Buf, BytesMut};
use log::{debug, warn};

use crate::types::Message;

struct Connection {
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
    in_buffer: BytesMut,
    // out_buffer: BytesMut,
}

#[derive(Debug, Clone, Copy)]
pub enum IRCError {
    ClientExited = -1,
    NoMessageLeftInBuffer = -2,
    LengthExceeded = -3,
}

impl Connection {
    fn new(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        return Connection {
            tcp_stream,
            socket_addr,
            in_buffer: BytesMut::with_capacity(1024 * 2),
            // out_buffer: BytesMut::with_capacity(1024 * 2),
        }
    }

    async fn read(&mut self) -> Result<Message, IRCError> {
        loop {
            match self.tcp_stream.read_buf(&mut self.in_buffer).await {
                Ok(0) => {
                    debug!("Remote side closed the session or the buffer is full");
                    self.shutdown().await;
                    return Err(IRCError::ClientExited);
                },
                Err(e) => {
                    warn!("{:}", e);
                    self.shutdown().await;
                    return Err(IRCError::ClientExited)
                },
                Ok(n) => {
                    debug!("read {:?} bytes from {:?}", n, self.socket_addr);
                },
            }

            match self.parse_frame() {
                Some(msg) => return Ok(msg),
                None => (),
            }
        }
    }

    async fn write(&mut self, msg: Message) -> Result<(), IRCError> {
        let msg_bytes = msg.to_bytes();
        let mut idx = 0;
        loop {
            match self.tcp_stream.write(&msg_bytes.as_bytes()[idx..]).await {
                Ok(0) => {
                    debug!("Remote side closed the session or the message is empty");
                    self.shutdown().await;
                    return Err(IRCError::ClientExited);
                },
                Err(e) => {
                    warn!("{:}", e);
                    self.shutdown().await;
                    return Err(IRCError::ClientExited)
                },
                Ok(n) => {
                    debug!("write {:?} bytes to {:?}", n, self.socket_addr);
                    idx += n;
                    if n == msg_bytes.len() {
                        return Ok(());
                    }
                },
            };
        }
    }

    async fn shutdown(&mut self) {
        _ = self.tcp_stream.shutdown().await
    }

    fn parse_frame(&mut self) -> Option<Message> {
        let mut cursor = Cursor::new(self.in_buffer.chunk());
        match Connection::get_frame(&mut cursor) {
            Some(msg_bytes) => {
                let frame_len = cursor.position() as usize;
                let opt_msg = Message::from_bytes(msg_bytes);
                self.in_buffer.advance(frame_len);
                return opt_msg
            }
            None => return None,
        };
    }

    fn get_frame<'a>(src: &mut Cursor<&'a [u8]>) -> Option<&'a [u8]> {
        if src.has_remaining() {
            let start = src.position() as usize;
            let end = src.get_ref().len();

            for i in start..end-1 {
                if src.get_ref()[i] == b'\r' && src.get_ref()[i+1] == b'\n' {
                    if i-start > 512 {
                        // TODO: off by one error ???
                        src.set_position((i+1)as u64);
                        return None;
                    }
                    src.set_position((i+1)as u64);
                    return Some(&src.get_ref()[start..i]);
                }
            }
            // NOTE: should buffer be advanced when no frame delimiter was read ?
            // src.set_position((end+1)as u64);
        }
        return None;
    }
}


#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use log::info;
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

    use crate::types::{Command, Message};

    use super::Connection;

    async fn start_listen() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        info!("Server listening on {}", addr);
        return (listener, addr);
    }

    #[tokio::test]
    async fn test_write() {
        let (listener, server_addr) = start_listen().await;
        let stream = TcpStream::connect(listener.local_addr().unwrap()).await.unwrap();
        let (mut server, client_addr) = listener.accept().await.unwrap();
        let mut client = Connection::new(stream, server_addr);
        info!("Client connected from {}", client_addr);

        let command = Command::PRIVMSG { targets: "#chan".to_string(), text: "Hello".to_string() };
        let message = Message{ tags:None, source:None, command };
        client.write(message).await.unwrap();
        client.shutdown().await;

        let mut res = String::new();
        server.read_to_string(&mut res).await.unwrap();

        assert_eq!("PRIVMSG #chan Hello\r\n", res);
        client.shutdown().await;
        server.shutdown().await.unwrap();
        drop(listener);
    }

    #[tokio::test]
    async fn test_read() {
        let (listener, server_addr) = start_listen().await;
        let stream = TcpStream::connect(listener.local_addr().unwrap()).await.unwrap();
        let (mut server, client_addr) = listener.accept().await.unwrap();
        let mut client = Connection::new(stream, server_addr);
        info!("Client connected from {}", client_addr);

        let _ = server.write_all(b"PRIVMSG #chan Hello\r\n").await;

        assert_eq!(b"PRIVMSG #chan Hello\r\n", client.read().await.unwrap().to_bytes().as_bytes());
        client.shutdown().await;
        server.shutdown().await.unwrap();
        drop(listener);
    }
}
