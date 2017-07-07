use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

use ipc_channel::ipc;

pub struct DuplexStream {
    pub tx: ipc::IpcSender<String>,
    pub rx: ipc::IpcReceiver<String>,
}

pub fn duplex() -> (DuplexStream, DuplexStream) {
    let (tx1, rx1) = ipc::channel().unwrap();
    let (tx2, rx2) = ipc::channel().unwrap();
    (DuplexStream { tx: tx1, rx: rx2 },
     DuplexStream { tx: tx2, rx: rx1 })
}