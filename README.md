# macOS timer app

a native-feeling macOS timer app with **Tauri** (Rust backend + vanilla JS frontend).

### What it does
- On launch a centered, borderless, always-on-top window appears with a **25:00** display and a **Start 25 min** button.
- The window has **no close button**, cannot be minimized/resized, and can only be dismissed by starting the timer.
- Clicking **Start** hides the window, creates a tray icon, and starts a 25-minute countdown shown in the menu bar.
- When the timer finishes, the window reappears automatically with a "Time's up" message; it’s again non-closable until you start another session.
- The tray icon menu has a **Quit** option to exit the app at any time.

### Built artifacts
- App bundle: [/Users/maksim.nesterenko/projects/macos-timer/src-tauri/target/release/bundle/macos/macos-timer.app](cci:9://file:///Users/maksim.nesterenko/projects/macos-timer/src-tauri/target/release/bundle/macos/macos-timer.app:0:0-0:0)
- Installer DMG: [/Users/maksim.nesterenko/projects/macos-timer/src-tauri/target/release/bundle/dmg/macos-timer_0.1.0_aarch64.dmg](cci:7://file:///Users/maksim.nesterenko/projects/macos-timer/src-tauri/target/release/bundle/dmg/macos-timer_0.1.0_aarch64.dmg:0:0-0:0)

Because the app is unsigned, you may need to right-click it and choose **Open** the first time.

### Key files
- Backend/timer/tray logic: `@/Users/maksim.nesterenko/projects/macos-timer/src-tauri/src/lib.rs:1-93`
- Window configuration: `@/Users/maksim.nesterenko/projects/macos-timer/src-tauri/tauri.conf.json:11-27`
- Frontend UI: `@/Users/maksim.nesterenko/projects/macos-timer/src/index.html:1-20`
- Frontend logic: `@/Users/maksim.nesterenko/projects/macos-timer/src/main.js:1-21`
- Styles: `@/Users/maksim.nesterenko/projects/macos-timer/src/styles.css:1-98`

### Run / rebuild
```zsh
cd /Users/maksim.nesterenko/projects/macos-timer
npm run tauri dev      # development mode
npm run tauri build    # release .app + .dmg
```

The 25-minute duration is set in `@/Users/maksim.nesterenko/projects/macos-timer/src-tauri/src/lib.rs:9-10`.
## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
