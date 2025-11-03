#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use winreg::enums::*;
use winreg::RegKey;
use std::io::Cursor;
use ico::IconDir;
use std::fs;
use std::process::{Command, Stdio};

const WUAUSERV: &str = "wuauserv";
const USOSVC: &str = "UsoSvc";
const WAASMEDIC: &str = "WaaSMedicSvc";
const BITS: &str = "BITS";
const DOSVC: &str = "DoSvc";


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
    let _ = stop_service(WUAUSERV);
    let _ = stop_service(USOSVC);
    let _ = stop_service(WAASMEDIC);
    let _ = stop_service(BITS);
    let _ = stop_service(DOSVC);

    let policies_ok = set_policies_disable();

    let _ = set_service_start(WUAUSERV, "disabled");
    let _ = set_service_start(USOSVC, "disabled");
    let _ = set_service_start(WAASMEDIC, "disabled");
    let _ = set_service_start(BITS, "disabled");
    let _ = set_service_start(DOSVC, "disabled");

    let _ = disable_update_tasks();

    policies_ok
}

fn restore_all() -> bool {
    let policies_ok = set_policies_enable();

    let _ = refresh_group_policy();

    let _ = set_service_start(WUAUSERV, "demand");
    let _ = set_service_start(USOSVC, "demand");
    let _ = set_service_start(WAASMEDIC, "demand");
    let _ = set_service_start(BITS, "demand");
    let _ = set_service_start(DOSVC, "auto");

    let _ = enable_update_tasks();

    policies_ok
}

fn set_policies_disable() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (wu_root, _) = match hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate") {
        Ok(x) => x,
        Err(_) => return false,
    };
    let (au, _) = match wu_root.create_subkey("AU") {
        Ok(x) => x,
        Err(_) => return false,
    };

    let mut ok = true;
    ok &= au.set_value("NoAutoUpdate", &1u32).is_ok();
    ok &= au.set_value("AUOptions", &1u32).is_ok();

    // WSUS redirection
    ok &= wu_root.set_value("WUServer", &"http://127.0.0.1").is_ok();
    ok &= wu_root.set_value("WUStatusServer", &"http://127.0.0.1").is_ok();
    ok &= wu_root.set_value("DoNotConnectToWindowsUpdateInternetLocations", &1u32).is_ok();
    ok &= wu_root.set_value("DisableDualScan", &1u32).is_ok();
    ok &= au.set_value("UseWUServer", &1u32).is_ok();

    ok
}

fn set_policies_enable() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let _ = hklm.delete_subkey_all(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate");
    let _ = hkcu.delete_subkey_all(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate");

    if let Ok(store) = hklm.open_subkey_with_flags(r"SOFTWARE\Policies\Microsoft\WindowsStore", KEY_ALL_ACCESS) {
        let _ = store.delete_value("RemoveWindowsStore");
        let _ = store.delete_value("DisableStoreApps");
        let _ = store.delete_value("AutoDownload");
    }

    if let Ok(wu_root) = hklm.open_subkey_with_flags(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate", KEY_ALL_ACCESS) {
        let _ = wu_root.delete_value("WUServer");
        let _ = wu_root.delete_value("WUStatusServer");
        let _ = wu_root.set_value("DoNotConnectToWindowsUpdateInternetLocations", &0u32);
        let _ = wu_root.set_value("DisableDualScan", &0u32);
        let _ = wu_root.delete_value("DisableWindowsUpdateAccess");
        let _ = wu_root.delete_value("SetDisableUXWUAccess");
        if let Ok(au) = wu_root.open_subkey_with_flags("AU", KEY_ALL_ACCESS) {
            let _ = au.set_value("NoAutoUpdate", &0u32);
            let _ = au.set_value("UseWUServer", &0u32);
            let _ = au.delete_value("AUOptions");
        }
    }
    true
}

fn refresh_group_policy() -> bool {
    let mut any = false;
    if run_cmd("gpupdate", &["/target:computer", "/force"]) {
        any = true;
    }
    if run_cmd("gpupdate", &["/target:user", "/force"]) {
        any = true;
    }
    any
}

fn _purge_local_gpo_files() -> bool {
    use std::path::Path;
    let mut any = false;
    let machine = r"C:\Windows\System32\GroupPolicy\Machine\Registry.pol";
    let user = r"C:\Windows\System32\GroupPolicy\User\Registry.pol";
    if Path::new(machine).exists() {
        if fs::remove_file(machine).is_ok() { any = true; }
    }
    if Path::new(user).exists() {
        if fs::remove_file(user).is_ok() { any = true; }
    }
    any
}

fn run_cmd(program: &str, args: &[&str]) -> bool {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn stop_service(name: &str) -> bool {
    run_cmd("sc", &["stop", name]) || run_cmd("net", &["stop", name])
}

fn _start_service(name: &str) -> bool {
    run_cmd("sc", &["start", name]) || run_cmd("net", &["start", name])
}

fn set_service_start(name: &str, mode: &str) -> bool {
    run_cmd("sc", &["config", name, "start=", mode])
}





fn disable_update_tasks() -> bool {
    let tasks = [
        r"\Microsoft\Windows\WindowsUpdate\Scheduled Start",
        r"\Microsoft\Windows\WindowsUpdate\AUScheduledInstall",
        r"\Microsoft\Windows\WindowsUpdate\Automatic App Update",
        r"\Microsoft\Windows\WindowsUpdate\Scheduled Start With Network",
        r"\Microsoft\Windows\WindowsUpdate\UPR",
        r"\Microsoft\Windows\UpdateOrchestrator\Schedule Scan",
        r"\Microsoft\Windows\UpdateOrchestrator\Schedule Scan Static Task",
        r"\Microsoft\Windows\UpdateOrchestrator\USO_UxBroker_Display",
        r"\Microsoft\Windows\UpdateOrchestrator\UpdateModelTask",
        r"\Microsoft\Windows\WaaSMedic\PerformRemediation",
    ];
    let mut any = false;
    for t in tasks {
        if run_cmd("schtasks", &["/Change", "/TN", t, "/DISABLE"]) {
            any = true;
        }
    }
    any
}

fn enable_update_tasks() -> bool {
    let tasks = [
        r"\Microsoft\Windows\WindowsUpdate\Scheduled Start",
        r"\Microsoft\Windows\WindowsUpdate\AUScheduledInstall",
        r"\Microsoft\Windows\WindowsUpdate\Automatic App Update",
        r"\Microsoft\Windows\WindowsUpdate\Scheduled Start With Network",
        r"\Microsoft\Windows\WindowsUpdate\UPR",
        r"\Microsoft\Windows\UpdateOrchestrator\Schedule Scan",
        r"\Microsoft\Windows\UpdateOrchestrator\Schedule Scan Static Task",
        r"\Microsoft\Windows\UpdateOrchestrator\USO_UxBroker_Display",
        r"\Microsoft\Windows\UpdateOrchestrator\UpdateModelTask",
        r"\Microsoft\Windows\WaaSMedic\PerformRemediation",
    ];
    let mut any = false;
    for t in tasks {
        if run_cmd("schtasks", &["/Change", "/TN", t, "/ENABLE"]) {
            any = true;
        }
    }
    any
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
