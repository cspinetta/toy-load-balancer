#![feature(mpsc_select)]

extern crate rand;

use std::io::{self, Write};
use std::sync::mpsc::{TryRecvError};
use server_manager::rand::Rng;
use ipc_channel::router::RouterProxy;
use ipc_channel::ipc::{IpcSender, IpcReceiverSet};
use std::sync::Arc;

use std::collections::HashMap;

pub struct HostResolver {
    next_host_id: usize,
    hosts: Vec<&'static str>
}

impl HostResolver {
    pub fn new() -> HostResolver {
        HostResolver { next_host_id: 0, hosts: vec!["http://127.0.0.1:3001"] }
    }

    pub fn get_next(&mut self) -> &'static str {
        let host = self.hosts[self.next_host_id];
        info!("Host selected: {}", host);
        self.next_host_id = (self.next_host_id + 1) % self.hosts.len();
        host
    }
}
