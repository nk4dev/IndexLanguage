//! IndexLanguage (il) — a TypeScript-flavored language that transpiles to JS.

mod ast;
mod checker;
mod codegen;
mod lexer;
mod parser;

pub const VERSION: &str = "0.1.0";

/// Compile IndexLanguage source to a JavaScript module string.
/// Returns the generated JS, or a multi-line diagnostic string on failure.
pub fn compile_source(src: &str) -> Result<String, String> {
    let tokens = lexer::lex(src)?;
    let program = parser::parse(tokens)?;
    checker::check(&program)?;
    Ok(codegen::emit(&program))
}

/// The `--help` / `--version` banner text, shared by the wasm entry point.
pub fn cli_text(arg: &str) -> String {
    match arg {
        "--version" | "-v" => format!("index lang v{VERSION}"),
        _ => [
            "index lang is a programming language for the web\u{1F4DA}",
            "",
            "usage: il <source.il>       compile to JavaScript",
            "       il --version         show compiler version",
            "       il --help            show this help",
            "",
            "types:    number  string  bool  void  any  T[]",
            "keywords: fn let const return if else while true false",
            "builtins: print(...)",
        ]
        .join("\n"),
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_api {
    use super::{cli_text, compile_source, VERSION};
    use wasm_bindgen::prelude::*;
    use web_sys::console::log_1;

    fn log(s: &str) {
        log_1(&JsValue::from_str(s));
    }

    /// Compile `src`, throwing a JS `Error` with diagnostics on failure.
    #[wasm_bindgen]
    pub fn compile(src: &str) -> Result<String, JsValue> {
        compile_source(src).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn version() -> String {
        VERSION.to_string()
    }

    /// Legacy entry point retained for the node example.
    #[wasm_bindgen]
    pub fn main(name: &str) {
        match name {
            "--help" | "-h" | "--version" | "-v" => log(&cli_text(name)),
            other => match compile_source(other) {
                Ok(js) => log(&js),
                Err(e) => log(&format!("compile error:\n{e}")),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compile_source;

    #[test]
    fn compiles_a_typed_function() {
        let src = r#"
            fn add(a: number, b: number): number {
                return a + b;
            }
            const total: number = add(2, 3);
            print(total);
        "#;
        let js = compile_source(src).expect("should compile");
        assert!(js.contains("function add(a, b)"));
        assert!(js.contains("const total = add(2, 3);"));
    }

    #[test]
    fn rejects_type_mismatch() {
        let src = r#"const x: number = "hi";"#;
        let err = compile_source(src).unwrap_err();
        assert!(err.contains("cannot assign string"));
    }

    #[test]
    fn rejects_bad_arity() {
        let src = r#"
            fn f(a: number): number { return a; }
            f(1, 2);
        "#;
        let err = compile_source(src).unwrap_err();
        assert!(err.contains("expects 1 argument"));
    }

    #[test]
    fn rejects_const_reassignment() {
        let src = r#"
            const x: number = 1;
            x = 2;
        "#;
        let err = compile_source(src).unwrap_err();
        assert!(err.contains("cannot reassign const"));
    }

    #[test]
    fn string_plus_number_coerces_to_string() {
        let src = r#"const s: string = "n=" + 1;"#;
        compile_source(src).expect("string + number should be allowed");
    }

    #[test]
    fn plus_on_bool_is_an_error() {
        let src = r#"const s: string = true + 1;"#;
        let err = compile_source(src).unwrap_err();
        assert!(err.contains("cannot apply '+'"));
    }

    #[test]
    fn control_flow_lowers_to_js() {
        let src = r#"
            fn fib(n: number): number {
                if n < 2 { return n; }
                let a: number = 0;
                let b: number = 1;
                let i: number = 2;
                while i <= n {
                    let next: number = a + b;
                    a = b;
                    b = next;
                    i = i + 1;
                }
                return b;
            }
            print(fib(10));
        "#;
        let js = compile_source(src).expect("should compile");
        assert!(js.contains("while ("));
        assert!(js.contains("function fib(n)"));
    }
}
