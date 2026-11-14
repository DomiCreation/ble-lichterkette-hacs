//! Lichterkette tester: control a Bluetooth light set from the Mac with the
//! same commands the Home Assistant integration sends.
//!
//!   lichterkette-tester                       open the window
//!   lichterkette-tester --screenshot out.png  draw ~120 frames, save a picture, exit

mod ble;
#[cfg(target_os = "macos")]
mod macos;
mod protocol;

use std::path::PathBuf;

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};

const BLUE_LEFT: Color32 = Color32::from_rgb(0x2b, 0x55, 0xa6);
const BLUE_RIGHT: Color32 = Color32::from_rgb(0x0a, 0x8b, 0xd2);
const BUTTON: Color32 = Color32::from_rgb(0xc3, 0xc8, 0xec);
const BUTTON_TEXT: Color32 = Color32::from_rgb(0x1d, 0x2a, 0x5c);
const SELECTED_TEXT: Color32 = Color32::from_rgb(0x0a, 0x5a, 0xa8);

struct App {
    ble: Option<ble::Ble>,
    selected: Option<usize>,
    program: String,
    level: u8,
    screenshot: Option<PathBuf>,
    frames: u32,
    shot_requested: bool,
    offline_status: String,
}

impl App {
    fn send(&mut self, commands: Vec<String>) {
        match (&self.ble, self.selected) {
            (Some(ble), Some(i)) => ble.send(ble::Cmd::Send(i, commands)),
            (Some(ble), None) => {
                ble.shared.lock().unwrap().status =
                    "Bitte zuerst eine Lichterkette suchen und auswählen.".into()
            }
            (None, _) => {}
        }
    }

    fn choose_program(&mut self, name: &str) {
        self.program = name.to_string();
        if name == protocol::STEADY {
            self.send(vec![protocol::steady_command(self.level)]);
        } else if let Some((_, cmd)) = protocol::PROGRAMS.iter().find(|(n, _)| *n == name) {
            self.send(vec![cmd.to_string()]);
        }
    }

    fn choose_level(&mut self, level: u8) {
        // Like the app: a brightness button switches to steady light at that level.
        self.level = level;
        self.program = protocol::STEADY.to_string();
        self.send(vec![protocol::steady_command(level)]);
    }
}

fn tile(ui: &mut egui::Ui, text: &str, selected: bool, fill: Color32, color: Color32, size: Vec2) -> bool {
    let (fill, color) = if selected { (Color32::WHITE, SELECTED_TEXT) } else { (fill, color) };
    let mut label = RichText::new(text).size(14.0).color(color);
    if selected {
        label = label.strong();
    }
    ui.add_sized(
        size,
        egui::Button::new(label)
            .fill(fill)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(8)),
    )
    .clicked()
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).size(13.0).strong().color(Color32::from_rgb(0xdc, 0xe8, 0xff)));
}

fn paint_gradient(ui: &egui::Ui) {
    let rect = ui.ctx().content_rect();
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), BLUE_LEFT);
    mesh.colored_vertex(rect.right_top(), BLUE_RIGHT);
    mesh.colored_vertex(rect.right_bottom(), BLUE_RIGHT);
    mesh.colored_vertex(rect.left_bottom(), BLUE_LEFT);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        paint_gradient(ui);

        let (status, devices, busy) = match &self.ble {
            Some(ble) => {
                let s = ble.shared.lock().unwrap();
                (s.status.clone(), s.devices.clone(), s.busy)
            }
            None => (self.offline_status.clone(), Vec::new(), false),
        };

        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(22, 18))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                let full = ui.available_width();

                ui.label(RichText::new("Lichterkette").size(26.0).color(Color32::WHITE));
                section(ui, "Lichterketten");

                egui::Frame::NONE
                    .fill(Color32::from_white_alpha(36))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_width(full - 16.0);
                        ui.set_height(104.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if devices.is_empty() {
                                ui.label(
                                    RichText::new("Noch keine Geräte – auf „Lichterketten suchen“ tippen.")
                                        .color(Color32::from_white_alpha(170)),
                                );
                            }
                            for (i, d) in devices.iter().enumerate() {
                                let text = format!(
                                    "{}{}   ({} dBm)",
                                    if d.likely { "★ " } else { "" },
                                    d.name,
                                    d.rssi
                                );
                                let chosen = self.selected == Some(i);
                                if ui
                                    .selectable_label(chosen, RichText::new(text).color(Color32::WHITE))
                                    .clicked()
                                {
                                    self.selected = Some(i);
                                }
                            }
                        });
                    });

                let scan_text = if busy { "Bitte warten …" } else { "Lichterketten suchen" };
                if tile(ui, scan_text, false, BUTTON, BUTTON_TEXT, Vec2::new(full, 40.0)) && !busy {
                    if let Some(ble) = &self.ble {
                        self.selected = None;
                        ble.send(ble::Cmd::Scan);
                    }
                }

                let half = (full - 8.0) / 2.0;
                ui.horizontal(|ui| {
                    if tile(ui, "Einschalten", false, Color32::from_rgb(0xe8, 0xf5, 0xe9),
                        Color32::from_rgb(0x1b, 0x5e, 0x20), Vec2::new(half, 40.0))
                    {
                        self.send(vec![protocol::CMD_ON.into()]);
                    }
                    if tile(ui, "Ausschalten", false, Color32::from_rgb(0xfd, 0xe7, 0xe7),
                        Color32::from_rgb(0x8a, 0x1c, 0x1c), Vec2::new(half, 40.0))
                    {
                        self.send(vec![protocol::CMD_OFF.into()]);
                    }
                });

                section(ui, "Lichtprogramme");
                let third = (full - 16.0) / 3.0;
                let mut names: Vec<&str> = protocol::PROGRAMS.iter().map(|(n, _)| *n).collect();
                names.push(protocol::STEADY);
                for row in names.chunks(3) {
                    ui.horizontal(|ui| {
                        for name in row {
                            let on = self.program == *name;
                            if tile(ui, name, on, BUTTON, BUTTON_TEXT, Vec2::new(third, 40.0)) {
                                self.choose_program(name);
                            }
                        }
                    });
                }

                section(ui, "Helligkeit (Dauerlicht)");
                let quarter = (full - 24.0) / 4.0;
                ui.horizontal(|ui| {
                    for (label, level) in protocol::BRIGHTNESS {
                        let on = self.level == level;
                        if tile(ui, label, on, BUTTON, BUTTON_TEXT, Vec2::new(quarter, 40.0)) {
                            self.choose_level(level);
                        }
                    }
                });

                ui.add_space(14.0);
                ui.label(RichText::new(status).size(13.0).color(Color32::from_rgb(0xe6, 0xf0, 0xff)));
            });

        // --screenshot: keep drawing (also when not focused), then save after ~120 frames.
        if let Some(path) = self.screenshot.clone() {
            self.frames += 1;
            ui.ctx().request_repaint();
            if self.frames >= 120 && !self.shot_requested {
                self.shot_requested = true;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Screenshot(
                    egui::ScreenshotCallback::new(move |image| {
                        let [w, h] = image.size;
                        let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                        let code = match image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                            Some(img) => {
                                if let Some(dir) = path.parent() {
                                    let _ = std::fs::create_dir_all(dir);
                                }
                                if img.save(&path).is_ok() { 0 } else { 1 }
                            }
                            None => 1,
                        };
                        std::process::exit(code);
                    }),
                ));
            }
        }
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    // Bluetooth permission on macOS: become our own responsible process first
    // (not needed for --screenshot, which never touches Bluetooth).
    #[cfg(target_os = "macos")]
    if screenshot.is_none() {
        if let Some(code) = macos::relaunch_disclaimed() {
            std::process::exit(code);
        }
    }

    // --ble-check: scan 3 s without a window and print what was found (for testing).
    if args.iter().any(|a| a == "--ble-check") {
        std::process::exit(ble::check());
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Lichterkette")
        .with_inner_size([460.0, 720.0])
        .with_min_inner_size([420.0, 640.0]);
    let mut options = eframe::NativeOptions::default();

    if screenshot.is_some() {
        // Open without taking focus, so a check never interrupts the person.
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
        #[cfg(target_os = "macos")]
        {
            options.event_loop_builder = Some(Box::new(|builder| {
                use winit::platform::macos::EventLoopBuilderExtMacOS;
                builder.with_activate_ignoring_other_apps(false);
            }));
        }
    }
    options.viewport = viewport;

    let shot = screenshot.clone();
    eframe::run_native(
        "Lichterkette",
        options,
        Box::new(move |cc| {
            // No Bluetooth in screenshot mode, so a check never asks for permission.
            let ble = if shot.is_none() { Some(ble::Ble::start(cc.egui_ctx.clone())) } else { None };
            Ok(Box::new(App {
                ble,
                selected: None,
                program: protocol::STEADY.to_string(),
                level: 3,
                screenshot: shot,
                frames: 0,
                shot_requested: false,
                offline_status: "Lichterkette einschalten, dann „Lichterketten suchen“.".into(),
            }))
        }),
    )
}
