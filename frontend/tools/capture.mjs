// Loads a page in headless Chrome, waits in real time, then prints the value
// of an optional expression and saves a screenshot. Real time matters: with
// Chrome's virtual time no animation frame runs, so neither Elm nor Three.js
// would draw anything.
//
// Usage: CHROME_PROFILE=<empty dir> node capture.mjs <url> <wait ms> <output png> [expression]
import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { setTimeout as sleep } from "node:timers/promises";

const [url, waitMilliseconds, outputPath, expression] = process.argv.slice(2);
const port = 9333;
const chrome = spawn("google-chrome", [
  "--headless=new", "--no-first-run", "--disable-extensions", "--use-angle=swiftshader",
  "--enable-unsafe-swiftshader", "--window-size=1100,860", `--remote-debugging-port=${port}`,
  `--user-data-dir=${process.env.CHROME_PROFILE}`, url,
], { stdio: "ignore" });
try {
  let target;
  for (let attempt = 0; attempt < 50 && !target; attempt++) {
    await sleep(200);
    try {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
      target = targets.find((candidate) => candidate.type === "page");
    } catch {}
  }
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve) => socket.addEventListener("open", resolve));
  let nextId = 1;
  const send = (method, params = {}) => new Promise((resolve) => {
    const id = nextId++;
    const listener = (event) => {
      const message = JSON.parse(event.data);
      if (message.id === id) { socket.removeEventListener("message", listener); resolve(message.result); }
    };
    socket.addEventListener("message", listener);
    socket.send(JSON.stringify({ id, method, params }));
  });
  await sleep(Number(waitMilliseconds));
  if (expression) {
    const evaluated = await send("Runtime.evaluate", { expression, returnByValue: true });
    console.log(JSON.stringify(evaluated.result?.value ?? evaluated));
  }
  const screenshot = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync(outputPath, Buffer.from(screenshot.data, "base64"));
} finally {
  chrome.kill();
}
