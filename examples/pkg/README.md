# IndexLanguage (`il`)

A small, statically-typed language that transpiles to readable ES2022
JavaScript. The type system is TypeScript-flavored and gradual. The compiler is
written in Rust and ships as WebAssembly.

See [`SPEC.md`](./SPEC.md) for the language reference.

```
fn fib(n: number): number {
  if n < 2 { return n; }
  return fib(n - 1) + fib(n - 2);
}

let i: number = 0;
while i < 10 {
  print("fib(" + i + ") = " + fib(i));
  i = i + 1;
}
```

## Requirements

- Rust + [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (global install recommended)
- [Bun](https://bun.sh) or Node 18+

## Build

```sh
bun run build        # wasm-pack, --target nodejs  -> pkg/
bun run build:web    # wasm-pack, --target web     -> examples/pkg/
```

## Use

**Node CLI**

```sh
node examples/index.js examples/samples/fib.il        # print compiled JS
node examples/index.js run examples/samples/fib.il    # compile and execute
node examples/index.js --version
```

**Browser playground**

```sh
bun run build:web
bun run serve        # serves . ; open /examples/
```

Edit `il` source on the left, see the compiled JavaScript on the right, and
click **Compile & Run** to execute it with output mirrored to the page.

## Test

```sh
cargo test           # lexer / parser / checker / codegen unit tests
```

## Compiler API

`pkg/il_compiler.js` exports:

- `compile(src: string): string` — returns generated JS, throws an `Error`
  whose message is the newline-separated diagnostics on failure.
- `version(): string`
- `main(arg: string): void` — legacy `--help` / `--version` / compile-and-log entry point
