use std::net::{SocketAddr, TcpStream, Shutdown};
use std::io::{Cursor, Read};

use bytes::{Buf, BytesMut};
use log::{debug, warn};

use crate::types::Message;

struct Conection {
    tcp_stream: TcpStream,
    socket_addr: SocketAddr,
    in_buffer: BytesMut,
    out_buffer: BytesMut,
}

#[derive(Debug, Clone, Copy)]
pub enum IRCError {
    ClientExited = -1,
    NoMessageLeftInBuffer = -2,
    LengthExceeded = -3,
}

impl Conection {
    fn new(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        return Conection {
            tcp_stream,
            socket_addr,
            in_buffer: BytesMut::with_capacity(1024 * 2),
            out_buffer: BytesMut::with_capacity(1024 * 2),
        }
    }

    fn read(&mut self) -> Result<Message, IRCError> {
        loop {
            match self.tcp_stream.read(&mut self.in_buffer) {
                Ok(0) => {
                    self.shutdown();
                    return Err(IRCError::ClientExited);
                },
                Ok(n) => debug!("read {:?} bytes from {:?}", n, self.socket_addr),
                Err(e) => {
                    warn!("{:}", e);
                    self.shutdown();
                    return Err(IRCError::ClientExited)
                },
            }

            match self.parse_frame() {
                Some(msg) => return Ok(msg),
                None => (),
            }
        }
    }

    fn write(&mut self, _msg: Message) -> Result<(), ()> {

        return Err(())
    }

    fn shutdown(&self) {
        _ = self.tcp_stream.shutdown(Shutdown::Both)
    }

    fn parse_frame(&mut self) -> Option<Message> {
        let mut cursor = Cursor::new(self.in_buffer.chunk());
        match get_frame(&mut cursor) {
            Some(msg_bytes) => {
                let frame_len = cursor.position() as usize;
                let opt_msg = Message::from_bytes(msg_bytes);
                self.in_buffer.advance(frame_len);
                return opt_msg
            }
            None => return None,
        };
    }
}

fn get_frame<'a>(src: &mut Cursor<&'a [u8]>) -> Option<&'a [u8]> {
    if src.has_remaining() {
        let start = src.position() as usize;
        let end = src.get_ref().len();

        for i in start..end-1 {
            // TODO: end frame on '\r' or '\n' or '\r\n'
            if src.get_ref()[i] == b'\r' && src.get_ref()[i+1] == b'\n' {
                if i-start > 512 {
                    // TODO: off by one error ???
                    src.set_position((i+2)as u64);
                    return None;
                }
                src.set_position((i+2)as u64);
                return Some(&src.get_ref()[start..i]);
            }
        }
        // NOTE: should buffer be advanced when no frame delimiter was read ?
        // src.set_position((end+1)as u64);
    }
    return None;
}
