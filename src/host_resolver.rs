#![feature(mpsc_select)]

extern crate rand;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct HostResolver {
    next_host_id: Arc<AtomicUsize>,
    hosts: Vec<String>
}

impl HostResolver {
    pub fn new(hosts: Vec<String>) -> HostResolver {
        info!("All available hosts: {:?}", hosts.clone());
        HostResolver { next_host_id: Arc::new(AtomicUsize::new(0)), hosts: hosts }
    }

    pub fn get_next(&self) -> String {
        let next = self.next_host_id.clone().fetch_add(1, Ordering::SeqCst);
        let idx = next % self.hosts.len();
        let host = self.hosts[idx].clone();
        info!("Host selected: {}", host);
        host
    }
}
