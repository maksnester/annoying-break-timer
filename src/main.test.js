import { beforeEach, describe, expect, it, vi } from "vitest";

function renderApp() {
  document.body.innerHTML = `
    <main class="container">
      <h1 id="title">Focus Timer</h1>
      <div id="timer-display">25:00</div>
      <p id="status">Start a 25-minute focus session.</p>
      <button id="start-btn" type="button">Start 25 min</button>
    </main>
  `;
}

let invokeMock;
let timerFinishedHandler;

beforeEach(async () => {
  vi.resetModules();
  renderApp();

  invokeMock = vi.fn().mockResolvedValue(undefined);
  timerFinishedHandler = null;

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
