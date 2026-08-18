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

#[derive(Clone, Debug, PartialEq)]
enum TimerCommand {
    Run,
    Pause,
    Restart,
    Stop,
}

#[derive(Debug, PartialEq)]
enum TickOutcome {
    Stopped,
    Paused,
    Finished,
    Continuing { remaining: u64, cmd: TimerCommand },
}

fn advance_tick(remaining: u64, cmd: TimerCommand) -> TickOutcome {
    match cmd {
        TimerCommand::Stop => TickOutcome::Stopped,
        TimerCommand::Pause => TickOutcome::Paused,
        TimerCommand::Restart => TickOutcome::Continuing {
            remaining: FOCUS_SECONDS,
            cmd: TimerCommand::Run,
        },
        TimerCommand::Run => {
            let remaining = remaining.saturating_sub(1);
            if remaining == 0 {
                TickOutcome::Finished
            } else {
                TickOutcome::Continuing {
                    remaining,
                    cmd: TimerCommand::Run,
                }
            }
        }
    }
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

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(TimerPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true,
        }
    })
}

#[cfg(target_os = "macos")]
fn make_window_appear_everywhere(window: &tauri::WebviewWindow) {
    use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt};

    if let Ok(panel) = window.to_panel::<TimerPanel>() {
        panel.set_level(PanelLevel::Floating.value());
        panel.set_collection_behavior(
            CollectionBehavior::new()
                .full_screen_auxiliary()
                .move_to_active_space()
                .value(),
        );
        panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    }
}

// `MoveToActiveSpace` only relocates the panel to the active Space when it is
// re-ordered front (macOS treats it and `CanJoinAllSpaces` as mutually exclusive,
// so we can't just be omnipresent). To make the panel follow the user across
// Space switches - including into a fullscreen app's own Space - while it's
// visible, we keep nudging it back to front on a short interval.
#[cfg(target_os = "macos")]
fn keep_panel_on_active_space(app: tauri::AppHandle) {
    use tauri_nspanel::ManagerExt;

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(400));
        let app_c = app.clone();
        app.run_on_main_thread(move || {
            if let Some(window) = app_c.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    if let Ok(panel) = app_c.get_webview_panel("main") {
                        panel.order_front_regardless();
                    }
                }
            }
        })
        .ok();
    });
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
            match advance_tick(remaining, cmd.clone()) {
                TickOutcome::Stopped => break,
                TickOutcome::Paused => continue,
                TickOutcome::Finished => {
                    let ctl_c = ctl.clone();
                    let app_c = app.clone();
                    app.run_on_main_thread(move || {
                        let tray = app_c.tray_by_id("main").expect("tray");
                        tray.set_title(Some("Time's up")).ok();
                        tray.set_tooltip(Some("Focus timer finished")).ok();
                        tray.set_visible(true).ok();

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
                TickOutcome::Continuing {
                    remaining: new_remaining,
                    cmd: new_cmd,
                } => {
                    remaining = new_remaining;
                    if new_cmd != cmd {
                        ctl.lock().unwrap().cmd = new_cmd;
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
            }
        }
    });
}

#[tauri::command]
fn start_timer(app: tauri::AppHandle, state: tauri::State<'_, TimerCtl>) {
    spawn_timer(app, state.inner().clone());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![start_timer]);

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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
            tray.set_title(Some("Focus Timer"))?;
            tray.set_visible(true)?;

            #[cfg(target_os = "macos")]
            {
                make_window_appear_everywhere(&app.get_webview_window("main").expect("main window"));
                keep_panel_on_active_space(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_pads_minutes_and_seconds() {
        assert_eq!(format_time(0), "00:00");
        assert_eq!(format_time(5), "00:05");
        assert_eq!(format_time(65), "01:05");
        assert_eq!(format_time(FOCUS_SECONDS), "25:00");
    }

    #[test]
    fn stop_ends_the_countdown() {
        assert_eq!(advance_tick(100, TimerCommand::Stop), TickOutcome::Stopped);
    }

    #[test]
    fn pause_keeps_the_remaining_time_unchanged() {
        assert_eq!(advance_tick(100, TimerCommand::Pause), TickOutcome::Paused);
    }

    #[test]
    fn restart_resets_remaining_time_and_resumes_running() {
        assert_eq!(
            advance_tick(3, TimerCommand::Restart),
            TickOutcome::Continuing {
                remaining: FOCUS_SECONDS,
                cmd: TimerCommand::Run,
            }
        );
    }

    #[test]
    fn run_decrements_remaining_time_by_one_second() {
        assert_eq!(
            advance_tick(10, TimerCommand::Run),
            TickOutcome::Continuing {
                remaining: 9,
                cmd: TimerCommand::Run,
            }
        );
    }

    #[test]
    fn run_finishes_when_the_last_second_elapses() {
        assert_eq!(advance_tick(1, TimerCommand::Run), TickOutcome::Finished);
    }
}
