import { beforeEach, describe, expect, it, vi } from "vitest";

function renderApp() {
  document.body.innerHTML = `
    <main class="container">
      <div id="main-view" class="view">
        <div class="header">
          <h1 id="title">Focus Timer</h1>
          <button id="settings-btn" type="button" aria-label="Settings">⚙</button>
        </div>
        <div id="timer-display">25:00</div>
        <p id="status">Start a 25-minute focus session.</p>
        <button id="start-btn" type="button">Start 25 min</button>
      </div>

      <div id="settings-view" class="view" hidden>
        <h1>Settings</h1>
        <label for="minutes-input">Focus minutes</label>
        <input id="minutes-input" type="number" min="1" value="25" />
        <button id="save-btn" type="button">Save</button>
      </div>
    </main>
  `;
}

let invokeMock;
let timerFinishedHandler;

beforeEach(async () => {
  vi.resetModules();
  renderApp();

  timerFinishedHandler = null;

  invokeMock = vi.fn((command) => {
    if (command === "get_settings") {
      return Promise.resolve({ focus_minutes: 25 });
    }
    return Promise.resolve();
  });

  window.__TAURI__ = {
    core: { invoke: invokeMock },
    event: {
      listen: (eventName, handler) => {
        if (eventName === "timer_finished") {
          timerFinishedHandler = handler;
        }
      },
    },
  };

  await import("./main.js");
});

describe("start button", () => {
  it("invokes start_timer and disables itself when clicked", async () => {
    const startBtn = document.querySelector("#start-btn");

    startBtn.click();
    await Promise.resolve();

    expect(invokeMock).toHaveBeenCalledWith("start_timer");
    expect(startBtn.disabled).toBe(true);
    expect(document.querySelector("#status").textContent).toBe(
      "Timer running... see the tray icon.",
    );
  });
});

describe("settings", () => {
  it("loads settings and shows the configured minutes on start", () => {
    expect(invokeMock).toHaveBeenCalledWith("get_settings");
    expect(document.querySelector("#timer-display").textContent).toBe("25:00");
    expect(document.querySelector("#start-btn").textContent).toBe("Start 25 min");
  });

  it("opens the settings view, saves new minutes, and updates the display", async () => {
    const settingsBtn = document.querySelector("#settings-btn");
    const saveBtn = document.querySelector("#save-btn");
    const minutesInput = document.querySelector("#minutes-input");

    settingsBtn.click();

    expect(document.querySelector("#main-view").hidden).toBe(true);
    expect(document.querySelector("#settings-view").hidden).toBe(false);

    minutesInput.value = "30";
    saveBtn.click();
    await Promise.resolve();

    expect(invokeMock).toHaveBeenCalledWith("set_settings", {
      settings: { focus_minutes: 30 },
    });
    expect(document.querySelector("#main-view").hidden).toBe(false);
    expect(document.querySelector("#settings-view").hidden).toBe(true);
    expect(document.querySelector("#timer-display").textContent).toBe("30:00");
    expect(document.querySelector("#start-btn").textContent).toBe("Start 30 min");
  });
});

describe("timer_finished event", () => {
  it("resets the display and re-enables the start button", () => {
    expect(timerFinishedHandler).toBeTypeOf("function");

    timerFinishedHandler();

    const startBtn = document.querySelector("#start-btn");
    expect(document.querySelector("#timer-display").textContent).toBe("25:00");
    expect(document.querySelector("#status").textContent).toBe(
      "Time's up! Start another session.",
    );
    expect(startBtn.textContent).toBe("Start another 25 min");
    expect(startBtn.disabled).toBe(false);
  });
});
