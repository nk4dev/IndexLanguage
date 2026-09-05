//! Hand-written lexer for IndexLanguage.

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Number(f64),
    Str(String),

    // keywords
    Fn,
    Let,
    Const,
    Return,
    If,
    Else,
    While,
    True,
    False,
    KwNumber,
    KwString,
    KwBool,
    KwVoid,
    KwAny,

    // punctuation / operators
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Dot,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    EqEq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    AmpAmp,
    PipePipe,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut out: Vec<Token> = Vec::new();

    while i < chars.len() {
        let c = chars[i];

        // whitespace
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // comments
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                if chars[i] == '\n' {
                    line += 1;
                }
                i += 1;
            }
            i += 2; // consume */
            continue;
        }

        // string literal
        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    let esc = chars[i + 1];
                    match esc {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                    i += 2;
                    continue;
                }
                if chars[i] == '\n' {
                    return Err(format!("line {}: unterminated string literal", line));
                }
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err(format!("line {}: unterminated string literal", line));
            }
            i += 1; // closing quote
            out.push(Token {
                tok: Tok::Str(s),
                line,
            });
            continue;
        }

        // number literal
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let value: f64 = text
                .parse()
                .map_err(|_| format!("line {}: invalid number '{}'", line, text))?;
            out.push(Token {
                tok: Tok::Number(value),
                line,
            });
            continue;
        }

        // identifier / keyword
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let tok = match text.as_str() {
                "fn" => Tok::Fn,
                "let" => Tok::Let,
                "const" => Tok::Const,
                "return" => Tok::Return,
                "if" => Tok::If,
                "else" => Tok::Else,
                "while" => Tok::While,
                "true" => Tok::True,
                "false" => Tok::False,
                "number" => Tok::KwNumber,
                "string" => Tok::KwString,
                "bool" => Tok::KwBool,
                "void" => Tok::KwVoid,
                "any" => Tok::KwAny,
                _ => Tok::Ident(text),
            };
            out.push(Token { tok, line });
            continue;
        }

        // operators / punctuation
        let two: String = if i + 1 < chars.len() {
            chars[i..i + 2].iter().collect()
        } else {
            String::new()
        };
        let (tok, len) = match two.as_str() {
            "==" => (Tok::EqEq, 2),
            "!=" => (Tok::NotEq, 2),
            "<=" => (Tok::Le, 2),
            ">=" => (Tok::Ge, 2),
            "&&" => (Tok::AmpAmp, 2),
            "||" => (Tok::PipePipe, 2),
            _ => {
                let t = match c {
                    '(' => Tok::LParen,
                    ')' => Tok::RParen,
                    '{' => Tok::LBrace,
                    '}' => Tok::RBrace,
                    '[' => Tok::LBracket,
                    ']' => Tok::RBracket,
                    ',' => Tok::Comma,
                    ';' => Tok::Semi,
                    ':' => Tok::Colon,
                    '.' => Tok::Dot,
                    '=' => Tok::Assign,
                    '+' => Tok::Plus,
                    '-' => Tok::Minus,
                    '*' => Tok::Star,
                    '/' => Tok::Slash,
                    '%' => Tok::Percent,
                    '!' => Tok::Bang,
                    '<' => Tok::Lt,
                    '>' => Tok::Gt,
                    other => {
                        return Err(format!("line {}: unexpected character '{}'", line, other))
                    }
                };
                (t, 1)
            }
        };
        out.push(Token { tok, line });
        i += len;
    }

    out.push(Token {
        tok: Tok::Eof,
        line,
    });
    Ok(out)
}
