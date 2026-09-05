import init, { main } from "https://apps.nknighta.me/IndexLanguage/examples/pkg/il_compiler.js";

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

init().then(() => {
  main("--help");
  main("--version");
});
