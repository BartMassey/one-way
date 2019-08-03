use crate::*;

use std::net::*;

impl GameHandle {
    pub fn run(self) -> ! {
        let listener = TcpListener::bind("127.0.0.1:10001").unwrap();
        loop {
            match listener.accept() {
                Ok((socket, addr)) => {
                    println!("new client: {:?}", addr);
                    let handle = self.clone();
                    let _ = std::thread::spawn(move || {
                        let reader = socket;
                        let writer = reader.try_clone().unwrap();
                        let reader = BufReader::new(reader);
                        handle.play(reader, writer);
                    });
                },
                Err(e) => {
                    println!("couldn't get client: {:?}", e);
                },
            }
        }
    }
}
