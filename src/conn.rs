// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use telnet::{
    Action::*,
    Event, Telnet, TelnetError,
    TelnetOption::{self, *},
};

use core::time::*;
use std::collections::HashSet;
use std::io::{self, ErrorKind, Write};
use std::net::*;

// Terminal type information from
// https://code.google.com/archive/p/bogboa/wikis/TerminalTypes.wiki
const TTYPES: &[&str] = &[
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

pub struct Connection {
    telnet: Telnet,
    ttypes: HashSet<String>,
    next_event: Option<Event>,
    timeout: Option<Duration>,
    pub cbreak: bool,
    pub echo: bool,
    pub ansi: bool,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

fn telnet_io_error(te: TelnetError) -> io::Error {
    io::Error::new(ErrorKind::Other, te)
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let telnet = Telnet::from_stream(Box::new(stream), 256);
        Connection {
            telnet,
            ttypes: HashSet::new(),
            next_event: None,
            timeout: None,
            cbreak: false,
            echo: true,
            ansi: false,
            width: None,
            height: None,
        }
    }

    pub fn negotiate_cbreak(&mut self) -> io::Result<bool> {
        self.telnet
            .negotiate(&Will, SuppressGoAhead)
            .map_err(telnet_io_error)?;
        let event = self.get_event()?;
        use Event::*;
        match event {
            Negotiation(Do, SuppressGoAhead) => {
                //eprintln!("terminal will cbreak");
                self.cbreak = true;
                Ok(true)
            }
            Negotiation(Dont, SuppressGoAhead) => {
                eprintln!("terminal wont cbreak");
                self.cbreak = false;
                Ok(false)
            }
            event => {
                self.next_event = Some(event);
                Ok(false)
            }
        }
    }

    pub fn negotiate_noecho(&mut self) -> io::Result<bool> {
        // XXX *We* will echo, so terminal should not.
        self.telnet
            .negotiate(&Will, Echo)
            .map_err(telnet_io_error)?;
        let event = self.get_event()?;
        use Event::*;
        match event {
            Negotiation(Do, Echo) => {
                //eprintln!("terminal wont echo");
                self.echo = false;
                Ok(true)
            }
            Negotiation(Dont, Echo) => {
                eprintln!("terminal will echo");
                self.echo = true;
                Ok(false)
            }
            event => {
                self.next_event = Some(event);
                Ok(false)
            }
        }
    }

    pub fn negotiate_ansi(&mut self) -> io::Result<bool> {
        self.telnet.negotiate(&Do, TTYPE).map_err(telnet_io_error)?;
        loop {
            let event = self.get_event()?;
            use Event::*;
            match event {
                Negotiation(Will, TTYPE) => {
                    //eprintln!("starting ANSI negotiation");
                    self.telnet
                        .subnegotiate(TelnetOption::TTYPE, &[SEND])
                        .map_err(telnet_io_error)?;
                }
                Negotiation(Wont, TTYPE) => {
                    //eprintln!("terminal wont ANSI");
                    self.ansi = false;
                    return Ok(false);
                }
                Subnegotiation(TTYPE, buf) => {
                    assert_eq!(buf[0], IS);
                    let ttype = std::str::from_utf8(&buf[1..]).unwrap().to_string();
                    for good_ttype in TTYPES {
                        let ttype = ttype.to_lowercase();
                        if ttype.starts_with(*good_ttype) {
                            //eprintln!("got ANSI terminal");
                            self.ansi = true;
                            return Ok(true);
                        }
                    }
                    if self.ttypes.contains(&ttype) {
                        //eprintln!("terminal cannot ANSI");
                        self.ansi = false;
                        return Ok(false);
                    }
                    //eprintln!("unloved terminal: {}", ttype);
                    self.ttypes.insert(ttype);
                    self.telnet
                        .subnegotiate(TTYPE, &[SEND])
                        .map_err(telnet_io_error)?;
                }
                event => {
                    self.next_event = Some(event);
                    return Ok(false);
                }
            }
        }
    }

    pub fn negotiate_winsize(&mut self) -> io::Result<bool> {
        self.telnet.negotiate(&Do, NAWS).map_err(telnet_io_error)?;
        loop {
            let event = self.get_event()?;
            use Event::*;
            match event {
                Negotiation(Will, NAWS) => {
                    //eprintln!("starting NAWS negotiation");
                    self.telnet
                        .subnegotiate(TelnetOption::NAWS, &[])
                        .map_err(telnet_io_error)?;
                }
                Negotiation(Wont, NAWS) => {
                    eprintln!("terminal wont NAWS");
                    self.width = None;
                    self.height = None;
                    return Ok(false);
                }
                Subnegotiation(NAWS, buf) => {
                    assert_eq!(buf.len(), 4);
                    #[allow(clippy::cast_lossless)]
                    let width: u16 = (buf[0] as u16) << 8 | buf[1] as u16;
                    #[allow(clippy::cast_lossless)]
                    let height: u16 = (buf[2] as u16) << 8 | buf[3] as u16;
                    //eprintln!("terminal winsize {} {}", width, height);
                    if width > 0 {
                        self.width = Some(width);
                    }
                    if height > 0 {
                        self.height = Some(height);
                    }
                    return Ok(width > 0 || height > 0);
                }
                event => {
                    self.next_event = Some(event);
                    return Ok(false);
                }
            }
        }
    }

    pub fn set_timeout(&mut self, ms: Option<u64>) {
        self.timeout = ms.map(Duration::from_millis);
    }

    fn get_event(&mut self) -> io::Result<Event> {
        if let Some(event) = self.next_event.take() {
            self.next_event = None;
            return Ok(event);
        }
        match self.timeout {
            None => self.telnet.read(),
            Some(timeout) => self.telnet.read_timeout(timeout),
        }
    }

    pub fn read(&mut self) -> io::Result<Option<String>> {
        loop {
            let event = self.get_event()?;
            use Event::*;
            match event {
                Data(buf) => match String::from_utf8(buf.to_vec()) {
                    Ok(s) => return Ok(Some(s)),
                    Err(e) => {
                        return Err(io::Error::new(ErrorKind::InvalidData, e));
                    }
                },
                TimedOut => return Ok(None),
                NoData => (),
                Error(err) => {
                    return Err(io::Error::new(ErrorKind::InvalidData, err));
                }
                Subnegotiation(subneg, buf) => {
                    eprintln!("telnet: unexpected subnegotiation: {:?} {:?}", subneg, buf)
                }
                neg => eprintln!("telnet: unexpected negotation: {:?}", neg),
            }
        }
    }

    pub fn listen<T>(runner: T)
    where
        T: RunConnection + Clone + Send + 'static,
    {
        let listener = TcpListener::bind("0.0.0.0:10001").unwrap();
        loop {
            match listener.accept() {
                Ok((mut socket, addr)) => {
                    println!("new client: {:?}", addr);
                    let runner = runner.clone();
                    let _ = std::thread::spawn(move || {
                        let mut conn = Connection::new(socket.try_clone().unwrap());
                        match conn.negotiate_winsize() {
                            Ok(true) => (),
                            Ok(false) => eprintln!("no winsize"),
                            Err(e) => eprintln!("no winsize: {}", e),
                        }
                        let termok = conn
                            .negotiate_cbreak()
                            .and_then(|_| conn.negotiate_noecho());
                        match termok {
                            Ok(true) => (),
                            e => {
                                eprintln!("cannot set up terminal: {:?}", e);
                                socket.write_all(
    b"Your telnet client cannot be put in no-echo single-character mode\n
     as needed to play the game. Apologies.\n").unwrap();
                                socket.flush().unwrap();
                                return;
                            }
                        }
                        // Don't currently need ANSI.
                        // assert!(conn.negotiate_ansi().unwrap());
                        conn.set_timeout(Some(100));
                        runner.run_connection(conn);
                    });
                }
                Err(e) => {
                    println!("couldn't get client: {:?}", e);
                }
            }
        }
    }
}

pub trait RunConnection {
    fn run_connection(self, conn: Connection);
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.telnet.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
