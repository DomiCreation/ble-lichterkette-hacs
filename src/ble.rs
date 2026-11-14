//! Bluetooth on its own thread. Like the app, the connection stays open.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::mpsc;

use crate::protocol;

#[derive(Clone)]
pub struct Found {
    pub name: String,
    pub rssi: i16,
    pub likely: bool,
}

#[derive(Default)]
pub struct Shared {
    pub status: String,
    pub devices: Vec<Found>,
    pub busy: bool,
}

pub enum Cmd {
    Scan,
    Send(usize, Vec<String>),
}

pub struct Ble {
    tx: mpsc::UnboundedSender<Cmd>,
    pub shared: Arc<Mutex<Shared>>,
}

impl Ble {
    pub fn start(ctx: eframe::egui::Context) -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            status: "Lichterkette einschalten, dann „Lichterketten suchen“.".into(),
            ..Default::default()
        }));
        let (tx, rx) = mpsc::unbounded_channel();
        let worker_shared = shared.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio");
            rt.block_on(worker(rx, worker_shared, ctx));
        });
        Self { tx, shared }
    }

    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }
}

/// Scan for 3 seconds without a window; prints the devices, returns an exit code.
pub fn check() -> i32 {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("tokio");
    rt.block_on(async {
        let central = match adapter().await {
            Ok(a) => a,
            Err(e) => {
                println!("Bluetooth nicht verfügbar: {e}");
                return 2;
            }
        };
        if let Err(e) = central.start_scan(ScanFilter::default()).await {
            println!("Suche fehlgeschlagen: {e}");
            return 3;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = central.stop_scan().await;
        let list = central.peripherals().await.unwrap_or_default();
        let mut likely = 0;
        for p in &list {
            if let Ok(Some(props)) = p.properties().await {
                if props.services.contains(&protocol::SERVICE) || props.services.contains(&protocol::ALT_SERVICE) {
                    likely += 1;
                    println!("★ {}", props.local_name.unwrap_or_else(|| "?".into()));
                }
            }
        }
        println!("{} Geräte gefunden, {likely} mit Lichterketten-Dienst", list.len());
        0
    })
}

fn set_status(shared: &Arc<Mutex<Shared>>, ctx: &eframe::egui::Context, text: impl Into<String>) {
    shared.lock().unwrap().status = text.into();
    ctx.request_repaint();
}

async fn adapter() -> Result<Adapter, String> {
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    manager
        .adapters()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| "Kein Bluetooth-Adapter gefunden".to_string())
}

async fn worker(
    mut rx: mpsc::UnboundedReceiver<Cmd>,
    shared: Arc<Mutex<Shared>>,
    ctx: eframe::egui::Context,
) {
    let mut found: Vec<Peripheral> = Vec::new();
    let mut adapter_cache: Option<Adapter> = None;

    while let Some(cmd) = rx.recv().await {
        shared.lock().unwrap().busy = true;
        ctx.request_repaint();
        if adapter_cache.is_none() {
            match adapter().await {
                Ok(a) => adapter_cache = Some(a),
                Err(e) => {
                    set_status(&shared, &ctx, format!("Bluetooth nicht verfügbar: {e}"));
                    shared.lock().unwrap().busy = false;
                    continue;
                }
            }
        }
        let central = adapter_cache.as_ref().unwrap();
        match cmd {
            Cmd::Scan => match scan(central, &shared, &ctx).await {
                Ok(list) => {
                    let rows = list.iter().map(|(_, f)| f.clone()).collect::<Vec<_>>();
                    found = list.into_iter().map(|(p, _)| p).collect();
                    let n = rows.len();
                    shared.lock().unwrap().devices = rows;

                    let mut auto_connected = false;
                    for (p, f) in &list {
                        if f.name.to_lowercase() == "light set" {
                            if p.is_connected().await.unwrap_or(false) {
                                set_status(&shared, &ctx, "Lichterkette 'Light set' bereits verbunden!".into());
                                auto_connected = true;
                            } else {
                                set_status(&shared, &ctx, "Verbinde automatisch mit 'Light set'…".into());
                                if let Err(e) = p.connect().await {
                                    set_status(&shared, &ctx, format!("Auto-Verbindung zu 'Light set' fehlgeschlagen: {e}"));
                                } else if let Err(e) = p.discover_services().await {
                                    set_status(&shared, &ctx, format!("Service-Entdeckung fehlgeschlagen: {e}"));
                                } else {
                                    set_status(&shared, &ctx, "Lichterkette 'Light set' verbunden!".into());
                                    auto_connected = true;
                                }
                            }
                            break;
                        }
                    }

                    if !auto_connected {
                        set_status(&shared, &ctx, format!("{n} Geräte gefunden (★ = Lichterketten-Dienst)", n));
                    }
                }
                Err(e) => set_status(&shared, &ctx, format!("Suche fehlgeschlagen: {e}")),
            },
            Cmd::Send(index, commands) => match found.get(index) {
                None => set_status(&shared, &ctx, "Bitte zuerst eine Lichterkette auswählen."),
                Some(p) => match send(p, &commands, &shared, &ctx).await {
                    Ok(()) => set_status(&shared, &ctx, format!("Gesendet: {}", commands.join(", "))),
                    Err(e) => set_status(&shared, &ctx, format!("Senden fehlgeschlagen: {e}")),
                },
            },
        }
        shared.lock().unwrap().busy = false;
        ctx.request_repaint();
    }
}

async fn scan(
    central: &Adapter,
    shared: &Arc<Mutex<Shared>>,
    ctx: &eframe::egui::Context,
) -> Result<Vec<(Peripheral, Found)>, String> {
    set_status(shared, ctx, "Suche 6 Sekunden nach Lichterketten …");
    central.start_scan(ScanFilter::default()).await.map_err(|e| e.to_string())?;
    tokio::time::sleep(Duration::from_secs(6)).await;
    let _ = central.stop_scan().await;
    let mut rows = Vec::new();
    for p in central.peripherals().await.map_err(|e| e.to_string())? {
        let Ok(Some(props)) = p.properties().await else { continue };
        let likely = props.services.contains(&protocol::SERVICE)
            || props.services.contains(&protocol::ALT_SERVICE);
        let found = Found {
            name: props.local_name.unwrap_or_else(|| "?".into()),
            rssi: props.rssi.unwrap_or(-127),
            likely,
        };
        rows.push((p, found));
    }
    rows.sort_by_key(|(_, f)| (!f.likely, -(f.rssi as i32)));
    Ok(rows)
}

async fn send(
    p: &Peripheral,
    commands: &[String],
    shared: &Arc<Mutex<Shared>>,
    ctx: &eframe::egui::Context,
) -> Result<(), String> {
    if !p.is_connected().await.unwrap_or(false) {
        set_status(shared, ctx, "Verbinde …");
        p.connect().await.map_err(|e| e.to_string())?;
        p.discover_services().await.map_err(|e| e.to_string())?;
    }
    let chars = p.characteristics();
    let ch = protocol::WRITE_CHARS
        .iter()
        .find_map(|u| chars.iter().find(|c| c.uuid == *u))
        .ok_or("Kein Befehls-Kanal 1001/2001 – ist das die Lichterkette?")?;
    for command in commands {
        let frame = protocol::build_frame(command, protocol::random_key())
            .ok_or("ungültiger Befehl")?;
        // The app writes without response (0x10001a0e4).
        p.write(ch, &frame, WriteType::WithoutResponse)
            .await
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}
