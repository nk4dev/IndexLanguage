import { readFileSync } from "node:fs";
import { compile, version } from "../pkg/il_compiler.js";

const args = process.argv.slice(2);

if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
  console.log("usage: node examples/index.js <source.il>   compile to JavaScript");
  console.log("       node examples/index.js run <source.il>   compile and execute");
  console.log("       node examples/index.js --version");
  process.exit(args.length === 0 ? 254 : 0);
}

if (args[0] === "--version" || args[0] === "-v") {
  console.log(`index lang v${version()}`);
  process.exit(0);
}

const shouldRun = args[0] === "run";
const file = shouldRun ? args[1] : args[0];

if (!file) {
  console.error("error: no source file given");
  process.exit(2);
}

let js;
try {
  js = compile(readFileSync(file, "utf8"));
} catch (e) {
  console.error(`compile error:\n${e.message ?? e}`);
  process.exit(1);
}

if (shouldRun) {
  new Function(js)();
} else {
  process.stdout.write(js);
}
