// Starts the Elm application and relays its backend calls to the Rust
// commands of the desktop shell.
const app = Elm.Main.init({ node: document.getElementById("app") });
const invoke = window.__TAURI__?.core?.invoke;

// Each request port carries a cell count; the matching result port receives
// `{ ok: value }` or `{ error: reason }`.
const relayedCommands = [
  { command: "draw_uniform_polycube", request: "requestDraw", result: "drawFinishedPort" },
  {
    command: "compare_growths",
    request: "requestGrowthComparison",
    result: "growthComparisonFinishedPort",
  },
  {
    command: "trace_enumeration",
    request: "requestEnumerationTrace",
    result: "enumerationTraceFinishedPort",
  },
  {
    command: "compare_closed_form",
    request: "requestClosedFormComparison",
    result: "closedFormComparisonFinishedPort",
  },
  { command: "build_gallery", request: "requestGallery", result: "galleryFinishedPort" },
  { command: "find_remarkable", request: "requestRemarkable", result: "remarkableFinishedPort" },
];

// Commands whose request port carries the whole argument object.
const relayedRequests = [
  {
    command: "run_automaton",
    request: "requestAutomatonRunPort",
    result: "automatonRunFinishedPort",
  },
  { command: "list_patterns", request: "requestPatternsPort", result: "patternsFinishedPort" },
  { command: "play_pattern", request: "requestPatternRunPort", result: "patternRunFinishedPort" },
];

function relay(command, request, result, toArguments) {
  app.ports[request].subscribe(async (payload) => {
    if (!invoke) {
      app.ports[result].send({
        error: "moteur de calcul absent, ouvrir cette page depuis l'application de bureau",
      });
      return;
    }
    try {
      app.ports[result].send({ ok: await invoke(command, toArguments(payload)) });
    } catch (error) {
      app.ports[result].send({ error: String(error) });
    }
  });
}

for (const { command, request, result } of relayedCommands) {
  relay(command, request, result, (cellCount) => ({ cellCount }));
}
for (const { command, request, result } of relayedRequests) {
  relay(command, request, result, (requestArguments) => requestArguments);
}

app.ports.ready.send(null);
