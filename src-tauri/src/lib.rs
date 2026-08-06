use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

const FOCUS_MINUTES: u64 = 25;
const FOCUS_SECONDS: u64 = FOCUS_MINUTES * 60;

#[derive(Clone, PartialEq)]
enum TimerCommand {
    Run,
    Pause,
    Restart,
    Stop,
}

struct TimerState {
    cmd: TimerCommand,
    last_title: String,
    pause_item: MenuItem<tauri::Wry>,
    restart_item: MenuItem<tauri::Wry>,
}

type TimerCtl = Arc<Mutex<TimerState>>;

fn format_time(total_seconds: u64) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn spawn_timer(app: tauri::AppHandle, ctl: TimerCtl) {
    let window = app.get_webview_window("main").expect("main window");
    window.hide().expect("hide window");

    {
        let mut state = ctl.lock().unwrap();
        state.cmd = TimerCommand::Run;
        let _ = state.pause_item.set_text("Pause");
        let _ = state.pause_item.set_enabled(true);
        let _ = state.restart_item.set_enabled(true);
    }

    let tray = app.tray_by_id("main").expect("tray");
    tray.set_title(Some(&format_time(FOCUS_SECONDS))).ok();

    thread::spawn(move || {
        let tick = Duration::from_secs(1);
        let mut remaining = FOCUS_SECONDS;

        loop {
            thread::sleep(tick);

            let cmd = ctl.lock().unwrap().cmd.clone();
            match cmd {
                TimerCommand::Stop => break,
                TimerCommand::Pause => continue,
                TimerCommand::Restart => {
                    remaining = FOCUS_SECONDS;
                    ctl.lock().unwrap().cmd = TimerCommand::Run;
                }
                TimerCommand::Run => {
                    remaining = remaining.saturating_sub(1);
                }
            }

            if remaining == 0 {
                let ctl_c = ctl.clone();
                let app_c = app.clone();
                app.run_on_main_thread(move || {
                    let tray = app_c.tray_by_id("main").expect("tray");
                    tray.set_title(Some("Time's up")).ok();
                    tray.set_tooltip(Some("Focus timer finished")).ok();

                    let state = ctl_c.lock().unwrap();
                    let _ = state.pause_item.set_enabled(false);
                    let _ = state.restart_item.set_enabled(false);

                    let window = app_c.get_webview_window("main").expect("main window");
                    window.show().ok();
                    window.set_focus().ok();

                    let _ = app_c.emit("timer_finished", ());
                })
                .ok();
                break;
            }

            let title = format_time(remaining);
            let ctl_c = ctl.clone();
            let app_c = app.clone();
            app.run_on_main_thread(move || {
                let mut state = ctl_c.lock().unwrap();
                state.last_title = title.clone();
                let tray = app_c.tray_by_id("main").expect("tray");
                tray.set_title(Some(&title)).ok();
            })
            .ok();
        }
    });
}

#[tauri::command]
fn start_timer(app: tauri::AppHandle, state: tauri::State<'_, TimerCtl>) {
    spawn_timer(app, state.inner().clone());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_timer])
        .setup(|app| {
            let pause_i = MenuItem::with_id(app, "pause", "Pause", false, None::<&str>)?;
            let restart_i = MenuItem::with_id(app, "restart", "Restart", false, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&pause_i, &restart_i, &quit_i])?;

            let ctl: TimerCtl = Arc::new(Mutex::new(TimerState {
                cmd: TimerCommand::Stop,
                last_title: String::new(),
                pause_item: pause_i.clone(),
                restart_item: restart_i.clone(),
            }));
            app.manage(ctl.clone());

            let ctl_for_tray = ctl.clone();
            let tray = TrayIconBuilder::with_id("main")
                .tooltip("Focus Timer")
                .icon(Image::new_owned(vec![0; 4], 1, 1))
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |_app, event| match event.id().as_ref() {
                    "quit" => _app.exit(0),
                    "pause" => {
                        let mut state = ctl_for_tray.lock().unwrap();
                        if state.cmd == TimerCommand::Pause {
                            state.cmd = TimerCommand::Run;
                            let _ = state.pause_item.set_text("Pause");
                            if let Some(tray) = _app.tray_by_id("main") {
                                tray.set_title(Some(&state.last_title)).ok();
                            }
                        } else if state.cmd == TimerCommand::Run {
                            state.cmd = TimerCommand::Pause;
                            let _ = state.pause_item.set_text("Resume");
                            if let Some(tray) = _app.tray_by_id("main") {
                                tray.set_title(Some(&format!("⏸ {}", state.last_title))).ok();
                            }
                        }
                    }
                    "restart" => {
                        let mut state = ctl_for_tray.lock().unwrap();
                        state.cmd = TimerCommand::Restart;
                        let _ = state.pause_item.set_text("Pause");
                        state.last_title = format_time(FOCUS_SECONDS);
                    }
                    _ => {}
                })
                .build(app)?;
            tray.set_icon(None)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
