# Annoying Break Timer

Something that reminds you to go get that damn break.

Written with **Tauri** (Rust backend + vanilla JS frontend).

### What it does
- On launch a centered, borderless, always-on-top window appears with a **25:00** display and a **Start 25 min** button.
- It appears across all you workspaces and full screen windows.
- The window has **no close button**, cannot be minimized/resized, and can only be dismissed by starting the timer.
- Clicking **Start** hides the window, creates a tray icon, and starts a 25-minute countdown shown in the menu bar.
- When the timer finishes, the window reappears automatically; it’s again non-closable until you start another session.
- The tray icon menu has a **Quit** option to exit the app at any time.
- You can customize the timer duration by clicking the cogwheel button on the main screen.

### Built artifacts
- App bundle: `./src-tauri/target/release/bundle/macos/annoying-break-timer.app`
- Installer DMG: `./src-tauri/target/release/bundle/dmg/annoying-break-timer_0.1.0_aarch64.dmg`

Because the app is unsigned, you may need to right-click it and choose **Open** the first time.

### Run / rebuild
```zsh
npm run tauri dev      # development mode
npm run build          # release .app + .dmg
```

