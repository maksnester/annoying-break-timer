use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{menu::MenuItem, Emitter, Manager};

pub const FOCUS_MINUTES: u64 = 25;
pub const FOCUS_SECONDS: u64 = FOCUS_MINUTES * 60;

#[derive(Clone, Debug, PartialEq)]
pub enum TimerCommand {
    Run,
    Pause,
    Restart,
    Stop,
}

#[derive(Debug, PartialEq)]
pub enum TickOutcome {
    Stopped,
    Paused,
    Finished,
    Continuing { remaining: u64, cmd: TimerCommand },
}

pub fn advance_tick(remaining: u64, cmd: TimerCommand) -> TickOutcome {
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

pub struct TimerState {
    pub cmd: TimerCommand,
    pub last_title: String,
    pub pause_item: MenuItem<tauri::Wry>,
    pub restart_item: MenuItem<tauri::Wry>,
}

pub type TimerCtl = Arc<Mutex<TimerState>>;

pub fn format_time(total_seconds: u64) -> String {
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
pub fn start_timer(app: tauri::AppHandle, state: tauri::State<'_, TimerCtl>) {
    spawn_timer(app, state.inner().clone());
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
