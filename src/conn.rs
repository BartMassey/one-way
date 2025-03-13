// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Handle a player connection, including telnet processing
//! and setup as well as starting game play.

use telnet::{
    Action::*,
    Event, Telnet, TelnetError,
    TelnetOption::{self, *},
};

use core::time::*;
#[cfg(feature = "ansi")]
use std::collections::HashSet;
use std::io::{self, ErrorKind, Write};
use std::net::*;

/// Terminal type information from
/// https://code.google.com/archive/p/bogboa/wikis/TerminalTypes.wiki
#[cfg(feature = "ansi")]
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

/// TTYPE subnegotiation commands.
#[cfg(feature = "ansi")]
const SEND: u8 = 1;
#[cfg(feature = "ansi")]
const IS: u8 = 0;

/// Connection state.
pub struct Connection {
    /// Telnet client instance.
    telnet: Telnet,
    /// Lookahead telnet event for telnet negotiation.
    next_event: Option<Event>,
    /// How long to wait for a command from the client.
    timeout: Option<Duration>,
    /// Terminal is cbreak.
    pub cbreak: bool,
    /// Terminal is will echo.
    pub echo: bool,
    /// Terminal is ansi.
    #[cfg(feature = "ansi")]
    pub ansi: bool,
    /// Terminal width.
    pub width: Option<u16>,
    /// Terminal height.
    pub height: Option<u16>,
}

/// Wrap a telnet error as an IO error.
fn telnet_io_error(te: TelnetError) -> io::Error {
    io::Error::new(ErrorKind::Other, te)
}

impl Connection {
    /// Make a new connection state for a stream.
    pub fn new(stream: TcpStream) -> Connection {
        let telnet = Telnet::from_stream(Box::new(stream), 256);
        Connection {
            telnet,
            next_event: None,
            timeout: None,
            cbreak: false,
            echo: true,
            #[cfg(feature = "ansi")]
            ansi: false,
            width: None,
            height: None,
        }
    }

    /// Turn on "cbreak": that is, make sure that each
    /// character typed in the telnet client is immediately
    /// sent to us.
    ///
    /// The SUPRESS-GO-AHEAD telnet option turns the client
    /// into a full-duplex terminal that does not wait for a
    /// server GO-AHEAD before sending characters.  Client
    /// characters are to be transmitted as soon as
    /// available.  See [RFC
    /// 858](https://datatracker.ietf.org/doc/html/rfc858).
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
                // Buffer peek.
                self.next_event = Some(event);
                Ok(false)
            }
        }
    }

    /// Turn on "noecho": that is, make sure that the client
    /// does not echo typed characters itself. This will allow
    /// us to send the game state without interference.
    ///
    /// The ECHO telnet option tells the client that *we*
    /// will echo characters so *they* should not. This is
    /// of course mightily confusing. See [RFC
    /// 857](https://datatracker.ietf.org/doc/html/rfc857).
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
                // Buffer peek.
                self.next_event = Some(event);
                Ok(false)
            }
        }
    }

    /// Negotiate a client terminal type with support for
    /// ANSI terminal escapes. This allows sophisticated
    /// cursor control and screen formatting.
    ///
    /// The telnet TERMINAL-TYPE (TTYPE) negotiation is a
    /// bit confusing, since the client can offer several
    /// possible terminal types to the server. We cycle
    /// through these until we find one that we "know"
    /// supports ANSI — see [TTYPES] above — or until we
    /// see the same terminal a second time.
    ///
    /// See also [RFC
    /// 1091](https://www.rfc-editor.org/rfc/rfc1091.html).
    #[cfg(feature = "ansi")]
    pub fn negotiate_ansi(&mut self) -> io::Result<bool> {
        // Seen terminal types for client. Used
        // for loop detection.
        let mut ttypes = HashSet::new();
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
                    // XXX This code is a mess, and needs miles of love.
                    assert_eq!(buf[0], IS);
                    let ttype = String::from_utf8_lossy(&buf[1..]).into_owned();
                    let ttype_lc = ttype.to_lowercase();

                    // Check terminal for ANSI-ness.
                    for good_ttype in TTYPES {
                        if ttype_lc.starts_with(*good_ttype) {
                            //eprintln!("got ANSI terminal");
                            self.ansi = true;
                            return Ok(true);
                        }
                    }

                    // Check for having cycled around.
                    if ttypes.contains(&ttype) {
                        //eprintln!("terminal cannot ANSI");
                        self.ansi = false;
                        return Ok(false);
                    }

                    // Remember the unwanted terminal type.
                    //eprintln!("unloved terminal: {}", ttype);
                    ttypes.insert(ttype);
                    self.telnet
                        .subnegotiate(TTYPE, &[SEND])
                        .map_err(telnet_io_error)?;
                }
                event => {
                    // Buffer peek.
                    self.next_event = Some(event);
                    return Ok(false);
                }
            }
        }
    }

    /// Get the width and height of the client terminal.
    /// This uses the telnet Negotiate About Window Size
    /// (NAWS) option. See [RFC
    /// 1073](https://datatracker.ietf.org/doc/html/rfc1073).
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
                    let width: u16 = ((buf[0] as u16) << 8) | buf[1] as u16;
                    let height: u16 = ((buf[2] as u16) << 8) | buf[3] as u16;
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
                    // Buffer peek.
                    self.next_event = Some(event);
                    return Ok(false);
                }
            }
        }
    }

    /// Set the connection read timeout in milliseconds (if
    /// `Some`) or clear the timeout (if `None).
    pub fn set_timeout(&mut self, ms: Option<u64>) {
        self.timeout = ms.map(Duration::from_millis);
    }

    /// Get the next telnet event, which may be from peek
    /// buffer or read.
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

    /// Read data or telnet in-band stuff from the client.
    /// Honor timeouts.
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

    /// Listen for client connections and attach them to the
    /// game via the given runner.
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

/// Can take the connection and use it in the game.
pub trait RunConnection {
    /// Take the connection and use it in the game.
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
