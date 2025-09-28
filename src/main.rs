#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use winreg::enums::*;
use winreg::RegKey;
use std::io::Cursor;
use ico::IconDir;
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

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
                let bg_color = if self.windows_update_disabled {
                    egui::Color32::from_rgb(0, 99, 177) //blue disabled
                } else {
                    egui::Color32::from_rgb(220, 53, 69) //red running
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
            small_label(ui, "- Corrupt wuauserv ImagePath");
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

    let corrupt_ok = corrupt_wuauserv();

    let policies_ok = set_policies_disable();

    let _ = set_service_start(WUAUSERV, "disabled");
    let _ = set_service_start(USOSVC, "disabled");
    let _ = set_service_start(WAASMEDIC, "disabled");
    let _ = set_service_start(BITS, "disabled");
    let _ = set_service_start(DOSVC, "disabled");

    let _ = disable_update_tasks();

    corrupt_ok || policies_ok
}

fn restore_all() -> bool {
    let policies_ok = set_policies_enable();

    let restore_ok = restore_wuauserv();

    let _ = set_service_start(WUAUSERV, "demand");
    let _ = set_service_start(USOSVC, "demand");
    let _ = set_service_start(WAASMEDIC, "demand");
    let _ = set_service_start(BITS, "demand");
    let _ = set_service_start(DOSVC, "auto");

    let _ = enable_update_tasks();

    policies_ok && restore_ok
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
    if let Ok(wu_root) = hklm.open_subkey_with_flags(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate", KEY_ALL_ACCESS) {
        let _ = wu_root.delete_value("WUServer");
        let _ = wu_root.delete_value("WUStatusServer");
        let _ = wu_root.set_value("DoNotConnectToWindowsUpdateInternetLocations", &0u32);
        let _ = wu_root.set_value("DisableDualScan", &0u32);
        let _ = wu_root.delete_subkey("AU");
    }
    
    if let Ok((wu_root, _)) = hklm.create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate") {
        if let Ok((au, _)) = wu_root.create_subkey("AU") {
            let _ = au.set_value("NoAutoUpdate", &0u32);
            let _ = au.set_value("UseWUServer", &0u32);
            let _ = au.delete_value("AUOptions");
        }
    }
    true
}

fn run_cmd_silent(program: &str, args: &[&str]) -> bool {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn stop_service(name: &str) -> bool {
    run_cmd_silent("sc", &["stop", name]) || run_cmd_silent("net", &["stop", name])
}

fn set_service_start(name: &str, mode: &str) -> bool {
    run_cmd_silent("sc", &["config", name, "start=", mode])
}

fn corrupt_wuauserv() -> bool {
    let _ = stop_service(WUAUSERV);

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok((key, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        let new_path = r"C:\WINDOWS\system32\svchostt.exe -k netsvcs -p"; // intentionally wrong
        let image_path_ok = key.set_value("ImagePath", &new_path).is_ok();
        let start_value_ok = key.set_value("Start", &4u32).is_ok(); // disabled
        return image_path_ok && start_value_ok;
    }
    false
}

fn restore_wuauserv() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok((key, _)) = hklm.create_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        let original_path = r"C:\WINDOWS\system32\svchost.exe -k netsvcs -p";
        let image_path_ok = key.set_value("ImagePath", &original_path).is_ok();
        let start_value_ok = key.set_value("Start", &3u32).is_ok(); // demand
        return image_path_ok && start_value_ok;
    }
    false
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
        if run_cmd_silent("schtasks", &["/Change", "/TN", t, "/DISABLE"]) {
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
        if run_cmd_silent("schtasks", &["/Change", "/TN", t, "/ENABLE"]) {
            any = true;
        }
    }
    any
}

fn read_status() -> bool {
    let policy_disabled = read_policy_no_auto_update();
    let wuauserv_disabled = read_wuauserv_is_corrupted_or_disabled();
    policy_disabled || wuauserv_disabled
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

fn read_wuauserv_is_corrupted_or_disabled() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Services\wuauserv") {
        let path_is_corrupt = key
            .get_value::<String, _>("ImagePath")
            .map(|p| !p.to_ascii_lowercase().contains("svchost.exe"))
            .unwrap_or(false);
        let start_is_disabled = key
            .get_value::<u32, _>("Start")
            .map(|v| v == 4)
            .unwrap_or(false);
        return path_is_corrupt || start_is_disabled;
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
