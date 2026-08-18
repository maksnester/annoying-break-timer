use std::thread;
use std::time::Duration;

use tauri::Manager;
use tauri_nspanel::{CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt};

tauri_nspanel::tauri_panel! {
    panel!(TimerPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true,
        }
    })
}

pub fn make_window_appear_everywhere(window: &tauri::WebviewWindow) {
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
pub fn keep_panel_on_active_space(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(800));
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
