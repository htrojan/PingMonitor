#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::fs;
use std::fs::File;
use std::future::Future;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Local, MIN_DATETIME, Utc};
use tauri::{command, State};
use tokio::io::AsyncWriteExt;
use std::sync::Mutex;
use winping::{AsyncPinger, Buffer, CreateError, Error, Pinger, PingFuture};
use std::io::Write;

#[derive(Copy, Clone)]
pub struct PingEntry {
    time: chrono::DateTime<Utc>,
    ping: u32,
}

impl PingEntry {
    pub fn empty() -> PingEntry {
        PingEntry{ time: MIN_DATETIME, ping: 0 }
    }
}

/// Stores PingData for the last 600 Entries
pub struct PingData {
    data: [PingEntry; 600],
    index_start: usize,
    size: usize
}

impl PingData {
    pub fn empty() -> PingData {
        PingData {
            data: [PingEntry::empty(); 600],
            index_start: 0,
            size: 0
        }
    }

    pub fn store_entry(&mut self, entry: PingEntry) {
        if self.size == 600 {
            // Find last entry in queue
            let tail_entry = (self.index_start + 600 - 1) % 600;
            self.data[tail_entry] = entry;
            self.index_start = tail_entry;

        } else {
            // Find index after this one
            let write_index = (self.index_start + 1) % 600;
            self.data[write_index] = entry;
            self.index_start = write_index;
        }
    }
}

#[command]
async fn test_ping() -> Result<String, String> {
    let pinger: AsyncPinger = AsyncPinger::new();
    let dst = String::from("127.0.0.1")
        .parse::<IpAddr>()
        .expect("Ip Address could not be parsed");

    let mut buffer = Buffer::new();
    match pinger.send(dst, buffer).await.result {
        Ok(rtt) => { Ok(rtt.to_string()) }
        Err(err) => { Err(err.to_string()) }
    }
}

fn ping() -> Result<u32, Error> {
    let pinger = match Pinger::new() {
        Ok(p) => {p}
        Err(e) => {return Err(Error::Other(256))}
    };

    let dst = String::from("2a00:1450:4001:829::2003")
        .parse::<IpAddr>()
        .expect("Ip Address could not be parsed");

    let mut buffer = Buffer::new();
    pinger.send(dst, &mut buffer)
}

async fn sync_ping(logger: Arc<Mutex<PingLog>>) {
    let time = Utc::now();
    let result = ping();
    match result{
        Ok(rtt) => {
            println!("{}: Ping is {}", Local::now() ,rtt);
            let entry = PingEntry{ time, ping: rtt };
            let mut logger = logger.lock().unwrap();
            logger.log(entry);
        },
        Err(_) => {eprint!("Error!\n")}
    }
}

async fn ping_loop() {
    let mut logger = Arc::new(Mutex::new(PingLog::new("pinglog.txt")));
    let ping_data = PingData::empty();
    loop {
        println!("Loop!");
        let ping_data = logger.clone();
        tauri::async_runtime::spawn(async move {
            sync_ping(ping_data).await;
        });
        tokio::time::sleep(Duration::from_secs(1)).await;

    }
}

struct PingLog {
    file: File
}

// unsafe impl Send for PingLog {}
impl PingLog {
    pub fn new(path: impl AsRef<Path>) -> PingLog {
        let file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        PingLog {
            file
        }
    }

    pub fn log(&mut self, entry: PingEntry) {
        let t = format!("{}, {}", entry.time, entry.ping);
        println!("{}", t);
        write!(&mut self.file, "{}\n", t);
    }
}

fn main() {
    tauri::async_runtime::spawn(ping_loop());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
