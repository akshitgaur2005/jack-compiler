// Keep your existing Token, TokenType, and Keyword structs. They are perfect.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Symbol(char),
    IntConst(u16),
    StrConst(String),
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Class, Constructor, Function, Method, Field, Static,
    Var, Int, Char, Boolean, Void, True, False, Null,
    This, Let, Do, If, Else, While, Return
}


pub fn tokenizer(content: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut line_number = 1;
    let chars = content.chars().collect::<Vec<char>>();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // 1. Handle Whitespace
        if c.is_whitespace() {
            if c == '\n' {
                line_number += 1;
            }
            i += 1;
            continue;
        }

        // 2. Handle Comments
        if c == '/' {
            if i + 1 < chars.len() {
                let next_char = chars[i + 1];
                if next_char == '/' { // Single-line comment
                    i += 2;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    continue; // Let the main loop handle the newline
                } else if next_char == '*' { // Multi-line comment
                    let start_line = line_number;
                    i += 2;
                    while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                        if chars[i] == '\n' {
                            line_number += 1;
                        }
                        i += 1;
                    }
                    if i + 1 >= chars.len() {
                        return Err(format!("Unterminated multi-line comment starting on line {}", start_line));
                    }
                    i += 2; // Consume "*/"
                    continue;
                }
            }
        }

        // 3. Handle Symbols
        if "{}()[].,;+-*/&|<>=~".contains(c) {
            tokens.push(Token {
                token_type: TokenType::Symbol(c),
                value: c.to_string(),
                line_number,
            });
            i += 1;
            continue;
        }

        // 4. Handle String Constants
        if c == '"' {
            i += 1; // Consume opening quote
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\n' {
                     return Err(format!("Unterminated string on line {}", line_number));
                }
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err(format!("Unterminated string on line {}", line_number));
            }
            i += 1; // Consume closing quote
            tokens.push(Token {
                token_type: TokenType::StrConst(s.clone()),
                value: s,
                line_number,
            });
            continue;
        }

        // 5. Handle Integer Constants
        if c.is_ascii_digit() {
            let mut num_str = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num_str.push(chars[i]);
                i += 1;
            }
            let value = num_str.parse::<u16>().map_err(|e| format!("Invalid integer '{}' on line {}: {}", num_str, line_number, e))?;
            tokens.push(Token {
                token_type: TokenType::IntConst(value),
                value: num_str,
                line_number,
            });
            continue;
        }

        // 6. Handle Keywords and Identifiers
        if c.is_alphabetic() || c == '_' {
            let mut identifier = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                identifier.push(chars[i]);
                i += 1;
            }
            let token_type = match identifier.as_str() {
                "class"     => TokenType::Keyword(Keyword::Class),
                "constructor" => TokenType::Keyword(Keyword::Constructor),
                "function"  => TokenType::Keyword(Keyword::Function),
                "method"    => TokenType::Keyword(Keyword::Method),
                "field"     => TokenType::Keyword(Keyword::Field),
                "static"    => TokenType::Keyword(Keyword::Static),
                "var"       => TokenType::Keyword(Keyword::Var),
                "int"       => TokenType::Keyword(Keyword::Int),
                "char"      => TokenType::Keyword(Keyword::Char),
                "boolean"   => TokenType::Keyword(Keyword::Boolean),
                "void"      => TokenType::Keyword(Keyword::Void),
                "true"      => TokenType::Keyword(Keyword::True),
                "false"     => TokenType::Keyword(Keyword::False),
                "null"      => TokenType::Keyword(Keyword::Null),
                "this"      => TokenType::Keyword(Keyword::This),
                "let"       => TokenType::Keyword(Keyword::Let),
                "do"        => TokenType::Keyword(Keyword::Do),
                "if"        => TokenType::Keyword(Keyword::If),
                "else"      => TokenType::Keyword(Keyword::Else),
                "while"     => TokenType::Keyword(Keyword::While),
                "return"    => TokenType::Keyword(Keyword::Return),
                _           => TokenType::Identifier(identifier.clone()),
            };
            tokens.push(Token {
                token_type,
                value: identifier,
                line_number,
            });
            continue;
        }

        // 7. Handle any other character
        return Err(format!("Invalid character '{}' on line {}", c, line_number));
    }

    Ok(tokens)
}
