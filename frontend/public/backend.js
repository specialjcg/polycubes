// Starts the Elm application and relays its backend calls to the Rust
// commands of the desktop shell.
const app = Elm.Main.init({ node: document.getElementById("app") });
const invoke = window.__TAURI__?.core?.invoke;

app.ports.requestDraw.subscribe(async (cellCount) => {
  if (!invoke) {
    app.ports.drawFinishedPort.send({
      error: "moteur de calcul absent, ouvrir cette page depuis l'application de bureau",
    });
    return;
  }
  try {
    const drawn = await invoke("draw_uniform_polycube", { cellCount });
    app.ports.drawFinishedPort.send({ drawn });
  } catch (error) {
    app.ports.drawFinishedPort.send({ error: String(error) });
  }
});

app.ports.ready.send(null);
