const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const mainView = document.querySelector("#main-view");
const settingsView = document.querySelector("#settings-view");
const displayEl = document.querySelector("#timer-display");
const statusEl = document.querySelector("#status");
const startBtn = document.querySelector("#start-btn");
const settingsBtn = document.querySelector("#settings-btn");
const minutesInput = document.querySelector("#minutes-input");
const saveBtn = document.querySelector("#save-btn");
const backBtn = document.querySelector("#back-btn");

let focusMinutes = 25;

function updateDisplay() {
  const minutes = String(focusMinutes).padStart(2, "0");
  displayEl.textContent = `${minutes}:00`;
  statusEl.textContent = `Start a ${focusMinutes}-minute focus session.`;
  startBtn.textContent = `Start ${focusMinutes} min`;
}

settingsBtn.addEventListener("click", () => {
  minutesInput.value = focusMinutes;
  mainView.hidden = true;
  settingsView.hidden = false;
});

backBtn.addEventListener("click", () => {
  mainView.hidden = false;
  settingsView.hidden = true;
});

saveBtn.addEventListener("click", async () => {
  const minutes = Number(minutesInput.value);
  if (!Number.isFinite(minutes) || minutes < 1) {
    statusEl.textContent = "Please enter a positive number of minutes.";
    return;
  }

  await invoke("set_settings", { settings: { focus_minutes: minutes } });
  focusMinutes = minutes;
  updateDisplay();
  mainView.hidden = false;
  settingsView.hidden = true;
});

startBtn.addEventListener("click", async () => {
  statusEl.textContent = "Timer running... see the tray icon.";
  startBtn.disabled = true;
  await invoke("start_timer");
});

listen("timer_finished", () => {
  updateDisplay();
  statusEl.textContent = "Time's up! Start another session.";
  startBtn.textContent = `Start another ${focusMinutes} min`;
  startBtn.disabled = false;
});

const settings = await invoke("get_settings");
focusMinutes = settings.focus_minutes;
updateDisplay();
