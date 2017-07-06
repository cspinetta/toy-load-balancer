use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

pub struct DuplexStream {
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

pub fn duplex() -> (DuplexStream, DuplexStream) {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    (DuplexStream { tx: tx1, rx: rx2 },
     DuplexStream { tx: tx2, rx: rx1 })
}