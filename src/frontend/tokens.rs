use super::Span;

#[derive(Debug)]
pub enum TokenData {
    Entity,
    Behaviour,
    InPorts,
    OutPorts,
    Literal(String),
    OpenCurly,
    CloseCurly,
    OpenParen,
    CloseParen,
    Comma,
    Colon,
    Semicolon,
    Assign,
    PortAssign,
}

#[derive(Debug)]
pub struct Token(pub TokenData, pub Span);

pub fn tokenize(span: Span) -> Vec<Token> {
    let content = span.content();
    let mut chars = content.chars().enumerate().peekable();

    let mut result = Vec::new();
    let mut start = 0;

    while let Some((i, tmp_c)) = chars.next() {
        match tmp_c {
            '<' => {
                let (_, peek_c) = chars.peek().unwrap();
                match peek_c {
                    '=' => {
                        result.push(Token(TokenData::PortAssign, span.sub_span(i..i + 2)));
                        let _ = chars.next();
                        start = i + 2;
                    }
                    other => {
                        dbg!(other);
                        todo!()
                    }
                };
            }
            '=' => {
                result.push(Token(TokenData::Assign, span.sub_span(i..i + 1)));
                start = i + 1;
            }
            '\n' | ' ' | ';' | ':' | ',' | '(' | ')' | '{' | '}' => {
                if i - start != 0 {
                    let inner = &content[start..i];
                    let inner_span = span.sub_span(start..i);
                    match inner {
                        "entity" => {
                            result.push(Token(TokenData::Entity, inner_span));
                        }
                        "behaviour" => {
                            result.push(Token(TokenData::Behaviour, inner_span));
                        }
                        "in_ports" => {
                            result.push(Token(TokenData::InPorts, inner_span));
                        }
                        "out_ports" => {
                            result.push(Token(TokenData::OutPorts, inner_span));
                        }
                        _ => {
                            result.push(Token(
                                TokenData::Literal(inner.to_string()),
                                span.sub_span(start..i),
                            ));
                        }
                    };
                }

                let seperator = &content[i..i + 1];
                match seperator {
                    " " => {}
                    "\n" => {}
                    ";" => {
                        result.push(Token(TokenData::Semicolon, span.sub_span(i..i + 1)));
                    }
                    ":" => {
                        result.push(Token(TokenData::Colon, span.sub_span(i..i + 1)));
                    }
                    "," => {
                        result.push(Token(TokenData::Comma, span.sub_span(i..i + 1)));
                    }
                    "(" => {
                        result.push(Token(TokenData::OpenParen, span.sub_span(i..i + 1)));
                    }
                    ")" => {
                        result.push(Token(TokenData::CloseParen, span.sub_span(i..i + 1)));
                    }
                    "{" => {
                        result.push(Token(TokenData::OpenCurly, span.sub_span(i..i + 1)));
                    }
                    "}" => {
                        result.push(Token(TokenData::CloseCurly, span.sub_span(i..i + 1)));
                    }
                    other => {
                        dbg!(other);
                        todo!("Unknown Seperator")
                    }
                };

                start = i + 1;
            }
            _ => {}
        };
    }

    result
}
