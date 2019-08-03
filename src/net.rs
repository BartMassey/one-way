use crate::*;

use telnet::*;
use NegotiationAction::*;
use TelnetOption::*;

use std::io::*;
use std::net::*;
use std::collections::HashSet;

// Terminal type information from
// https://code.google.com/archive/p/bogboa/wikis/TerminalTypes.wiki
const TTYPES: &[&'static str] = &[
    "ansi",
    "xterm",
    "eterm",
    "rxvt",
    "tintin++",
    "gosclient",
    "mushclient",
    "zmud",
    "gosclient",
    "vt1",
    "tinyfugue",
];

// TTYPE subnegotiation commands.
const SEND: u8 = 1;
const IS: u8 = 0;

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

pub struct Connection {
    telnet: Telnet,
    ttypes: HashSet<String>,
    pub ansi: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let mut telnet = Telnet::from_stream(Box::new(stream), 256);
        telnet.negotiate(Will, SuppressGoAhead);
        telnet.negotiate(Will, Echo);
        telnet.negotiate(Do, TTYPE);
        Connection { telnet, ttypes: HashSet::new(), ansi: false }
    }

    fn negotiate_ansi(&mut self) {
        self.telnet.subnegotiate(TelnetOption::TTYPE, &[SEND]);
    }

    pub fn read(&mut self) -> io::Result<String> {
        'eventloop: loop {
            let event = self.telnet.read()?;
            use TelnetEvent::*;
            match event {
                Data(buf) => match String::from_utf8(buf.to_vec()) {
                    Ok(s) => return Ok(s),
                    Err(e) => {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            e,
                        ))
                    }
                },
                TimedOut => panic!("unexpected telnet read timeout"),
                NoData => eprintln!("unexpected telnet read nodata"),
                Error(msg) => {
                    panic!("unexpected telnet read error: {}", msg)
                }
                Negotiation(Do, SuppressGoAhead) => (),
                Negotiation(Do, Echo) => (),
                Negotiation(Will, TTYPE) => self.negotiate_ansi(),
                Negotiation(Wont, TTYPE) => eprintln!("terminal wont ANSI"),
                Subnegotiation(TTYPE, buf) => {
                    assert_eq!(buf[0], IS);
                    let ttype = std::str::from_utf8(&buf[1..]).unwrap().to_string();
                    for good_ttype in TTYPES {
                        let ttype = ttype.to_lowercase();
                        if ttype.len() < good_ttype.len() {
                            continue;
                        }
                        if &ttype[0..good_ttype.len()] == *good_ttype {
                            eprintln!("got ANSI terminal");
                            self.ansi = true;
                            continue 'eventloop;
                        }
                    }
                    if self.ttypes.contains(&ttype) {
                        eprintln!("terminal cannot ANSI");
                        continue 'eventloop;
                    }
                    eprintln!("unloved terminal: {}", ttype);
                    self.ttypes.insert(ttype);
                    self.telnet.subnegotiate(TTYPE, &[SEND]);
                },
                neg => eprintln!("unexpected negotation: {:?}", neg),
            }
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.telnet.write(buf)
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
                }
                Err(e) => {
                    println!("couldn't get client: {:?}", e);
                }
            }
        }
    }
}
