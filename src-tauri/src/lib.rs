use std::thread;
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

const FOCUS_MINUTES: u64 = 25;
const FOCUS_SECONDS: u64 = FOCUS_MINUTES * 60;

fn format_time(total_seconds: u64) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[tauri::command]
fn start_timer(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").expect("main window");
    window.hide().expect("hide window");

    let end = Instant::now() + Duration::from_secs(FOCUS_SECONDS);
    let tray = app.tray_by_id("main").expect("tray");
    tray.set_title(Some(&format_time(FOCUS_SECONDS))).ok();

    thread::spawn(move || {
        let tick = Duration::from_secs(1);

        loop {
            let now = Instant::now();

            if now >= end {
                let app_for_thread = app.clone();
                app.run_on_main_thread(move || {
                    let tray = app_for_thread.tray_by_id("main").expect("tray");
                    tray.set_title(Some("Time's up")).ok();
                    tray.set_tooltip(Some("Focus timer finished")).ok();

                    let window = app_for_thread
                        .get_webview_window("main")
                        .expect("main window");
                    window.show().ok();
                    window.set_focus().ok();

                    let _ = app_for_thread.emit("timer_finished", ());
                })
                .ok();
                break;
            }

            let remaining = (end - now).as_secs();
            let title = format_time(remaining);
            let app_for_thread = app.clone();
            app.run_on_main_thread(move || {
                let tray = app_for_thread.tray_by_id("main").expect("tray");
                tray.set_title(Some(&title)).ok();
            })
            .ok();

            thread::sleep(tick);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_timer])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::with_id("main")
                .tooltip("Focus Timer")
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("app icon"),
                )
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
