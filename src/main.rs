#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use winreg::enums::*;
use winreg::RegKey;
use std::io::Cursor;
use ico::IconDir;
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn get_disable_script() -> &'static str {
    include_str!("../disable.ps1")
}

fn get_enable_script() -> &'static str {
    include_str!("../enable.ps1")
}


struct MyApp {
    windows_update_disabled: bool,
    status_message: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NoUpdater");
            ui.add_space(10.0);

            let mut toggle_clicked = false;

            ui.horizontal(|ui| {
                // Custom toggle switch
                let toggle_size = egui::Vec2::new(50.0, 25.0);
                let (rect, response) = ui.allocate_exact_size(toggle_size, egui::Sense::click());

                if response.clicked() {
                    toggle_clicked = true;
                }

                // Draw the toggle switch background
                let hovered = response.hovered();
                let bg_color = if self.windows_update_disabled {
                    if hovered {
                        egui::Color32::from_rgb(0, 79, 140) // darker blue when hovered
                    } else {
                        egui::Color32::from_rgb(0, 99, 177) // blue disabled
                    }
                } else {
                    if hovered {
                        egui::Color32::from_rgb(176, 42, 55) // darker red when hovered
                    } else {
                        egui::Color32::from_rgb(220, 53, 69) // red running
                    }
                };

                ui.painter().rect_filled(rect, 12.5, bg_color);

                // Draw the toggle circle
                let circle_radius = 10.0;
                let circle_center = if self.windows_update_disabled {
                    egui::Pos2::new(rect.max.x - circle_radius - 2.5, rect.center().y)
                } else {
                    egui::Pos2::new(rect.min.x + circle_radius + 2.5, rect.center().y)
                };

                ui.painter().circle_filled(circle_center, circle_radius, egui::Color32::WHITE);

                ui.add_space(10.0);

                if self.windows_update_disabled {
                    ui.label(egui::RichText::new("Disabled").color(egui::Color32::from_rgb(0, 99, 177)));
                } else {
                    ui.label(egui::RichText::new("Running").color(egui::Color32::from_rgb(220, 53, 69)));
                }
            });

            if toggle_clicked {
                let target_state = !self.windows_update_disabled;

                if target_state {
                    if enforce_disable_all() {
                        self.windows_update_disabled = true;
                        self.status_message = "Windows Update disabled".to_string();
                    } else {
                        self.status_message = "Failed to disable (run as admin)".to_string();
                    }
                } else {
                    if restore_all() {
                        self.windows_update_disabled = false;
                        self.status_message = "Windows Update restored".to_string();
                    } else {
                        self.status_message = "Failed to restore (run as admin)".to_string();
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();

            fn small_label(ui: &mut egui::Ui, text: &str) {
            ui.label(egui::RichText::new(text).size(11.0));
            }

            ui.label("This software does:");
            small_label(ui, "- Service disable (wuauserv, UsoSvc, WaaSMedicSvc, BITS, DoSvc)");
            small_label(ui, "- Policies: NoAutoUpdate, WSUS redirection, block WU internet");
            small_label(ui, "- Disable Update Orchestrator and WaaS Medic tasks");


            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
            }
        });
    }
}

fn main() -> eframe::Result<()> {

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 230.0])
            .with_resizable(false)
            .with_icon(load_app_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "NoUpdater",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp {
            windows_update_disabled: read_status(),
            status_message: String::new(),
        }))),
    )
}


fn enforce_disable_all() -> bool {
    run_powershell_script(get_disable_script())
}

fn restore_all() -> bool {
    run_powershell_script(get_enable_script())
}















fn read_status() -> bool {
    read_policy_no_auto_update()
}

fn read_policy_no_auto_update() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(au) = hklm.open_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU") {
        if let Ok(no_auto) = au.get_value::<u32, _>("NoAutoUpdate") {
            return no_auto == 1;
        }
    }
    false
}



fn run_powershell_script(script: &str) -> bool {
    let mut cmd = Command::new("powershell");
    cmd.arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd.status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn load_app_icon() -> egui::IconData {
    let bytes: &[u8] = include_bytes!("../icon.ico");
    let cursor = Cursor::new(bytes);

    let dir = IconDir::read(cursor).expect("Failed to read icon directory");
    let first = dir.entries().first().expect("No icon entries found");
    let img = first.decode().expect("Failed to decode icon");

    egui::IconData {
        rgba: img.rgba_data().to_vec(),
        width: img.width(),
        height: img.height(),
    }
}
