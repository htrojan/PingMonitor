#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::future::Future;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Local, MIN_DATETIME, Utc};
use tauri::{App, AppHandle, command, Manager, State, Wry};
use tokio::io::AsyncWriteExt;
use std::sync::Mutex;
use winping::{AsyncPinger, Buffer, CreateError, Error, Pinger, PingFuture};
use std::io::Write;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Serialize)]
pub struct PingEntry {
    ping: u32,
    time: chrono::DateTime<Utc>,
}

impl PingEntry {
    pub fn empty() -> PingEntry {
        PingEntry{ time: MIN_DATETIME, ping: 0 }
    }
}

pub struct PingBuffer {
    data: Vec<PingEntry>,
    index_start: usize,
}

impl PingBuffer {
    pub fn empty() -> PingBuffer {
        PingBuffer {
            data: Vec::with_capacity(120),
            index_start: 0,
        }
    }

    pub fn store_entry(&mut self, entry: PingEntry) {
        self.data.push(entry)
    }

    /// Converts the current buffer into a json string and
    /// deletes the buffers contents
    pub fn send_json(&mut self) -> String{
        let data = serde_json::to_string(&self.data);
        data.unwrap()
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

async fn sync_ping(logger: Arc<Mutex<PingLog>>, ping_data: Arc<Mutex<PingBuffer>>, app: AppHandle<Wry>) {
    let time = Utc::now();
    let result = ping();
    match result{
        Ok(rtt) => {
            let entry = PingEntry{ time, ping: rtt };
            let mut logger = logger.lock().unwrap();
            logger.log(entry.clone());
            // let mut ping_data = ping_data.lock().unwrap();
            // ping_data.store_entry(entry);
            app.emit_all("ping".into(), &entry);
        },
        Err(_) => {eprint!("Error!\n")}
    }
}

async fn ping_loop(app: AppHandle<Wry>) {
    let logger = Arc::new(Mutex::new(PingLog::new("pinglog.txt")));
    let ping_data = Arc::new(Mutex::new(PingBuffer::empty()));

    loop {
        // println!("Loop!");
        let ping_data = ping_data.clone();
        let logger = logger.clone();
        let app2 = app.clone();
        tauri::async_runtime::spawn(async move {
            sync_ping(logger.clone(), ping_data.clone(), app2).await;
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
        // println!("{}", t);
        write!(&mut self.file, "{}\n", t);
    }
}

fn main() {
    tauri::Builder::default()
        // .invoke_handler(tauri::generate_handler![test_ping])
        .setup(|app| {
            let app2 = app.handle().clone();
            tauri::async_runtime::spawn(ping_loop(app2));
            println!("Setup events");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
