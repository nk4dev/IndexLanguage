import init, { main } from "./pkg/il_compiler.js";

// Mirror console output onto the page.
const out = document.getElementById("console");

function mirror(kind) {
  const original = console[kind].bind(console);
  return (...args) => {
    original(...args);
    if (!out) return;
    const line = document.createElement("div");
    line.className = kind;
    line.textContent = args
      .map((a) => (typeof a === "string" ? a : JSON.stringify(a)))
      .join(" ");
    out.appendChild(line);
  };
}

console.log = mirror("log");
console.warn = mirror("warn");
console.error = mirror("error");

const argsInput = document.getElementById("args");
const runButton = document.getElementById("run");

function run() {
  const arg = argsInput.value.trim();
  if (!arg) {
    console.warn("Please provide an argument.");
    return;
  }
  console.log(`$ il ${arg}`);
  main(arg);
}

init().then(() => {
  runButton.disabled = false;
  runButton.addEventListener("click", run);
  argsInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") run();
  });
});
