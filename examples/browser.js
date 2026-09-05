import init, { compile, version } from "./pkg/il_compiler.js";

const out = document.getElementById("console");
const sourceEl = document.getElementById("source");
const outputEl = document.getElementById("output");
const runButton = document.getElementById("run");
const compileButton = document.getElementById("compile");
const clearButton = document.getElementById("clear");
const verEl = document.getElementById("ver");

function append(kind, text) {
  const line = document.createElement("div");
  line.className = kind;
  line.textContent = text;
  out.appendChild(line);
  out.scrollTop = out.scrollHeight;
}

// Mirror console output from executed programs onto the page.
function mirror(kind) {
  const original = console[kind].bind(console);
  return (...args) => {
    original(...args);
    append(
      kind,
      args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" "),
    );
  };
}
console.log = mirror("log");
console.warn = mirror("warn");
console.error = mirror("error");

function compileSource() {
  const src = sourceEl.value;
  try {
    const js = compile(src);
    outputEl.textContent = js;
    return js;
  } catch (e) {
    const msg = String(e && e.message ? e.message : e);
    outputEl.textContent = msg;
    append("error", msg);
    return null;
  }
}

function run() {
  const js = compileSource();
  if (js == null) return;
  append("cmd", "$ il run");
  try {
    // Strip the "use strict" module wrapper concerns by running as a function body.
    new Function(js)();
  } catch (e) {
    append("error", String(e && e.stack ? e.stack : e));
  }
}

clearButton.addEventListener("click", () => {
  out.textContent = "";
});

init().then(() => {
  verEl.textContent = "v" + version();
  runButton.disabled = false;
  compileButton.disabled = false;
  runButton.addEventListener("click", run);
  compileButton.addEventListener("click", compileSource);
  compileSource();
});
