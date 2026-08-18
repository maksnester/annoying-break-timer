mod config;
mod timer;
mod tray;

#[cfg(target_os = "macos")]
mod panel;

use tauri::Manager;
use config::{get_settings, set_settings};
use timer::start_timer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![start_timer, get_settings, set_settings]);

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup_tray(app)?;

            #[cfg(target_os = "macos")]
            {
                panel::make_window_appear_everywhere(&app.get_webview_window("main").expect("main window"));
                panel::keep_panel_on_active_space(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
