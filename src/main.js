const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const displayEl = document.querySelector("#timer-display");
const statusEl = document.querySelector("#status");
const startBtn = document.querySelector("#start-btn");
const titleEl = document.querySelector("#title");

startBtn.addEventListener("click", async () => {
  statusEl.textContent = "Timer running... see the tray icon.";
  startBtn.disabled = true;
  await invoke("start_timer");
});

listen("timer_finished", () => {
  displayEl.textContent = "25:00";
  statusEl.textContent = "Time's up! Start another session.";
  startBtn.textContent = "Start another 25 min";
  startBtn.disabled = false;
});
