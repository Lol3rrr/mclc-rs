use super::tokens::{Token, TokenData};
use super::Span;

mod scopeiter;
use scopeiter::*;

#[derive(Debug)]
pub enum Error {
    MissingEntityName,
    UnexpectedToken {
        expected: Vec<TokenNames>,
        got: Token,
    },
}

#[derive(Debug)]
pub enum TokenNames {
    Literal,
    InPorts,
    OutPorts,
    Behaviour,
    OpenCurly,
}

fn parse_args(tokens: &mut dyn Iterator<Item = Token>) -> Vec<Token> {
    let mut result = Vec::new();

    while let Some(tok) = tokens.next() {
        match &tok.0 {
            TokenData::CloseParen => break,
            TokenData::Literal(_) => {
                result.push(tok);

                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::Comma => {}
                    TokenData::CloseParen => break,
                    other => panic!("Unexpected Token: {:?}", other),
                };
            }
            other => panic!("Unexpected arg: {:?}", other),
        };
    }

    result
}

#[derive(Debug)]
pub struct Entity {
    pub name: Span,
    pub in_ports: Vec<(Token, Token)>,
    pub out_ports: Vec<(Token, Token)>,
    pub behaviour: Vec<BehaviourStatement>,
}

fn parse_ports<I>(mut tokens: ScopeIter<I>) -> Vec<(Token, Token)>
where
    I: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    while let Some(tok) = tokens.next() {
        match &tok.0 {
            TokenData::Literal(_) => {
                let name_tok = tok;

                let colon_tok = tokens.next().unwrap();
                match colon_tok.0 {
                    TokenData::Colon => {}
                    other => panic!("Expected :, got {:?}", other),
                };

                let ty_tok = tokens.next().unwrap();

                let semicolon_tok = tokens.next().unwrap();
                match semicolon_tok.0 {
                    TokenData::Semicolon => {}
                    other => panic!("Expected ;, got {:?}", other),
                };

                result.push((name_tok, ty_tok));
            }
            other => panic!("Expected Port-Name or ;, got {:?}", other),
        };
    }

    result
}

#[derive(Debug)]
pub enum BehaviourValue {
    Operation { name: Token, arguments: Vec<Token> },
    Variables { vars: Vec<Token> },
}

fn parse_value(tokens: &mut dyn Iterator<Item = Token>) -> BehaviourValue {
    let init_token = tokens.next().unwrap();

    match &init_token.0 {
        TokenData::Literal(_) => {
            let name_tok = init_token;

            let opening_tok = tokens.next().unwrap();
            match opening_tok.0 {
                TokenData::OpenParen => {}
                other => panic!("Unexpected Token: {:?}", other),
            };

            let args = parse_args(tokens);

            BehaviourValue::Operation {
                name: name_tok,
                arguments: args,
            }
        }
        TokenData::OpenParen => {
            let args = parse_args(tokens);

            BehaviourValue::Variables { vars: args }
        }
        other => panic!("Unexpected Token: {:?}", other),
    }
}

#[derive(Debug)]
pub enum BehaviourStatement {
    PortAssign {
        targets: Vec<Token>,
        value: BehaviourValue,
    },
    VarAssign {
        targets: Vec<Token>,
        value: BehaviourValue,
    },
}

fn parse_behaviour<I>(mut tokens: ScopeIter<I>) -> Vec<BehaviourStatement>
where
    I: Iterator<Item = Token>,
{
    let mut result = Vec::new();

    while let Some(tok) = tokens.next() {
        match tok.0 {
            TokenData::OpenParen => {
                let targets = parse_args(&mut tokens);

                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::Assign => {
                        let value = parse_value(&mut tokens);

                        let ending_tok = tokens.next().unwrap();
                        match ending_tok.0 {
                            TokenData::Semicolon => {}
                            other => panic!("Expected Semicolon, got {:?}", other),
                        };

                        result.push(BehaviourStatement::VarAssign { targets, value });
                    }
                    TokenData::PortAssign => {
                        let value = parse_value(&mut tokens);

                        let ending_tok = tokens.next().unwrap();
                        match ending_tok.0 {
                            TokenData::Semicolon => {}
                            other => panic!("Expected Semicolon, got {:?}", other),
                        };

                        result.push(BehaviourStatement::PortAssign { targets, value });
                    }
                    other => panic!("Unexpected Token: {:?}", other),
                };
            }
            other => panic!("Unexpected Token: {:?}", other),
        };
    }

    result
}

fn parse_entity<I>(name: Span, mut tokens: ScopeIter<I>) -> Result<Entity, Error>
where
    I: Iterator<Item = Token>,
{
    let mut entity = Entity {
        name,
        in_ports: Vec::new(),
        out_ports: Vec::new(),
        behaviour: Vec::new(),
    };

    while let Some(tok) = tokens.next() {
        match tok.0 {
            TokenData::InPorts => {
                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::OpenCurly => {}
                    other => panic!("Expected Open Curly, got {:?}", other),
                };

                let tokens = ScopeIter::new(tokens.by_ref());
                let in_ports = parse_ports(tokens);

                entity.in_ports = in_ports;
            }
            TokenData::OutPorts => {
                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::OpenCurly => {}
                    other => panic!("Expected Open Curly, got {:?}", other),
                };

                let tokens = ScopeIter::new(tokens.by_ref());
                let out_ports = parse_ports(tokens);

                entity.out_ports = out_ports;
            }
            TokenData::Behaviour => {
                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::OpenCurly => {}
                    other => panic!("Expected Open Curly, got {:?}", other),
                };

                let tokens = ScopeIter::new(tokens.by_ref());
                let behaviour = parse_behaviour(tokens);

                entity.behaviour = behaviour;
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: vec![
                        TokenNames::InPorts,
                        TokenNames::OutPorts,
                        TokenNames::Behaviour,
                    ],
                    got: tok,
                })
            }
        };
    }

    Ok(entity)
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Entity>, Error> {
    let mut tokens = tokens.into_iter();

    let mut entities = Vec::new();

    while let Some(tok) = tokens.next() {
        match tok.0 {
            TokenData::Entity => {
                let name_tok = tokens.next().ok_or(Error::MissingEntityName)?;
                let entity_name = match name_tok.0 {
                    TokenData::Literal(_) => name_tok.1,
                    _ => {
                        return Err(Error::UnexpectedToken {
                            expected: vec![TokenNames::Literal],
                            got: name_tok,
                        })
                    }
                };

                let next_tok = tokens.next().unwrap();
                match next_tok.0 {
                    TokenData::OpenCurly => {}
                    _ => {
                        return Err(Error::UnexpectedToken {
                            expected: vec![TokenNames::OpenCurly],
                            got: next_tok,
                        })
                    }
                };

                let in_scope_iter = ScopeIter::new(tokens.by_ref());
                let entity = parse_entity(entity_name, in_scope_iter)?;

                entities.push(entity);
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    Ok(entities)
}
