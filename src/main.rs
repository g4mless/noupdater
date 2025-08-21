#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use winreg::enums::*;
use winreg::RegKey;

struct MyApp {
    windows_update_disabled: bool,
    status_message: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Update Fucker");
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
                let _visuals = ui.style().interact(&response);
                let bg_color = if self.windows_update_disabled {
                    egui::Color32::from_rgb(40, 167, 69) // Green when enabled
                } else {
                    egui::Color32::from_rgb(220, 53, 69) // Red when disabled
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
                
                // Add labels for the toggle states
                if self.windows_update_disabled {
                    ui.label(egui::RichText::new("Fucked (Disabled)").color(egui::Color32::from_rgb(40, 167, 69)));
                } else {
                    ui.label(egui::RichText::new("Not Fucked (Running)").color(egui::Color32::from_rgb(220, 53, 69)));
                }
            });

            // Apply changes when toggle is clicked
            if toggle_clicked {
                let target_state = !self.windows_update_disabled;
                
                if target_state {
                    // Trying to fuck Windows Update
                    if corrupt_wuauserv() {
                        self.windows_update_disabled = true;
                        self.status_message = "Windows Update disabled".to_string();
                    } else {
                        self.status_message = "Failed to disable (pls run as admin)".to_string();
                    }
                } else {
                    // Trying to restore Windows Update
                    // (why should restore this asshole service that took a lot of my data package when i hotspot my laptop)
                    if restore_wuauserv() {
                        self.windows_update_disabled = false;
                        self.status_message = "Windows Update restored".to_string();
                    } else {
                        self.status_message = "Failed to restore (pls run as admin)".to_string();
                    }
                }
            }

            // Show status message
            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
            }
        });
    }
}

fn corrupt_wuauserv() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok((key, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        let new_path = r"C:\WINDOWS\system32\svchostt.exe -k netsvcs -p"; // corrupt
        return key.set_value("ImagePath", &new_path).is_ok();
    }
    false
}

fn restore_wuauserv() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok((key, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        let original_path = r"C:\WINDOWS\system32\svchost.exe -k netsvcs -p"; // restore
        return key.set_value("ImagePath", &original_path).is_ok();
    }
    false
}

fn read_status() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        if let Ok(path) = key.get_value::<String, _>("ImagePath") {
            return path.contains("svchostt.exe"); // fuck this
        }
    }
    false
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([280.0, 180.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Update Fucker",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp {
            windows_update_disabled: read_status(),
            status_message: String::new(),
        }))),
    )
}
