use std::sync::{Arc, Mutex};

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager,
};

use crate::config::Settings;
use crate::timer::{format_time, TimerCommand, TimerCtl, TimerState};

pub fn setup_tray(app: &tauri::App) -> tauri::Result<TrayIcon> {
    let settings = Settings::load(&app.handle());

    let pause_i = MenuItem::with_id(app, "pause", "Pause", false, None::<&str>)?;
    let restart_i = MenuItem::with_id(app, "restart", "Restart", false, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&pause_i, &restart_i, &quit_i])?;

    let ctl: TimerCtl = Arc::new(Mutex::new(TimerState {
        cmd: TimerCommand::Stop,
        last_title: String::new(),
        focus_minutes: settings.focus_minutes,
        pause_item: pause_i.clone(),
        restart_item: restart_i.clone(),
    }));
    app.manage(ctl.clone());

    let tray = TrayIconBuilder::with_id("main")
        .tooltip("Focus Timer")
        .icon(Image::new_owned(vec![0; 4], 1, 1))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| handle_menu_event(app, event.id().as_ref(), &ctl))
        .build(app)?;
    tray.set_icon(None)?;
    tray.set_title(Some("Focus Timer"))?;
    tray.set_visible(true)?;

    Ok(tray)
}

fn handle_menu_event(app: &AppHandle, id: &str, ctl: &TimerCtl) {
    match id {
        "quit" => app.exit(0),
        "pause" => toggle_pause(app, ctl),
        "restart" => request_restart(ctl),
        _ => {}
    }
}

fn toggle_pause(app: &AppHandle, ctl: &TimerCtl) {
    let mut state = ctl.lock().unwrap();
    if state.cmd == TimerCommand::Pause {
        state.cmd = TimerCommand::Run;
        let _ = state.pause_item.set_text("Pause");
        if let Some(tray) = app.tray_by_id("main") {
            tray.set_title(Some(&state.last_title)).ok();
        }
    } else if state.cmd == TimerCommand::Run {
        state.cmd = TimerCommand::Pause;
        let _ = state.pause_item.set_text("Resume");
        if let Some(tray) = app.tray_by_id("main") {
            tray.set_title(Some(&format!("⏸ {}", state.last_title))).ok();
        }
    }
}

fn request_restart(ctl: &TimerCtl) {
    let mut state = ctl.lock().unwrap();
    state.cmd = TimerCommand::Restart;
    let _ = state.pause_item.set_text("Pause");
    state.last_title = format_time(state.focus_minutes * 60);
}
