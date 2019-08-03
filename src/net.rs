use crate::*;

use telnet::*;

use std::net::*;
use std::io::*;

#[derive(Debug)]
pub struct NegotiationError;

impl std::fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "negotiation error")
    }
}

impl std::error::Error for NegotiationError {
    fn description(&self) -> &str {
        "negotiation not expected"
    }
}

pub struct Connection(Telnet);

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection(Telnet::from_stream(Box::new(stream), 256))
    }

    pub fn read(&mut self) -> io::Result<String> {
        use TelnetEvent::*;
        match self.0.read()? {
            Data(buf) => match String::from_utf8(buf.to_vec()) {
                Ok(s) => Ok(s),
                Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e)),
            },
            TimedOut => panic!("unexpected telnet read timeout"),
            NoData => panic!("unexpected telnet read nodata"),
            Error(msg) => panic!("unexpected telnet read error: {}", msg),
            neg => {
                eprintln!("{:?}", neg);
                Err(io::Error::new(ErrorKind::InvalidData, NegotiationError))
            }
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl GameHandle {
    pub fn run(self) -> ! {
        let listener = TcpListener::bind("127.0.0.1:10001").unwrap();
        loop {
            match listener.accept() {
                Ok((socket, addr)) => {
                    println!("new client: {:?}", addr);
                    let handle = self.clone();
                    let _ = std::thread::spawn(move || {
                        handle.play(Connection::new(socket));
                    });
                },
                Err(e) => {
                    println!("couldn't get client: {:?}", e);
                },
            }
        }
    }
}
