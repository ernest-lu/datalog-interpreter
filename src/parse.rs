use either::Either;

use logos::{Lexer, Logos};
use std::result::Result;

#[derive(Logos, Debug)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[token(".decl")]
    DeclHeader,

    #[token(".rule")]
    RuleHeader,

    #[token(".input")]
    Input,

    #[token(".output")]
    Output,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    #[token(",")]
    Comma,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token(";")]
    Semicolon,

    #[token(":-")]
    Implies,

    #[regex(r"[0-9]+", |lex| lex.slice().to_owned().parse::<u32>().unwrap())]
    Number(u32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeclKind {
    Input,
    Output,
}

pub trait FactLike {
    fn name(&self) -> &str;
    fn params(&self) -> &Vec<String>;
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub params: Vec<String>,
    pub kind: DeclKind,
}
impl FactLike for Declaration {
    fn name(&self) -> &str {
        &self.name
    }
    fn params(&self) -> &Vec<String> {
        &self.params
    }
}

fn parse_params(lexer: &mut Lexer<'_, Token>) -> Result<Vec<String>, String> {
    let mut params = vec![];
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Ident(ident)) => {
                params.push(ident);
            }
            Ok(Token::Comma) => {
                continue;
            }
            Ok(Token::ParenClose) => {
                break;
            }
            _ => {
                return Err(format!("{:?} is not a valid token in parse_params", token));
            }
        }
    }
    Ok(params)
}

// Parse a declaration with or without an the input flag.
fn parse_declaration_or_fact(
    lexer: &mut Lexer<'_, Token>,
    with_input: bool,
    is_declaration: bool,
) -> Result<Either<Declaration, Fact>, String> {
    // .decl <ident>(<params>) .<input/output>
    let mut name = String::new();
    let mut params = vec![];
    let mut kind = DeclKind::Input;

    while let Some(token) = lexer.next() {
        if !is_declaration {
            dbg!(&token);
        }
        match token {
            Ok(Token::Ident(ident)) => {
                name = ident;
            }
            Ok(Token::ParenOpen) => {
                params = parse_params(lexer)?;
            }
            Ok(Token::Input) => {
                kind = DeclKind::Input;
            }
            Ok(Token::Output) => {
                kind = DeclKind::Output;
            }
            Ok(Token::Semicolon) => {
                break;
            }
            Ok(Token::Implies) => {
                if with_input {
                    return Err("Input flag not allowed after :-".to_string());
                } else {
                    // without input done here
                    break;
                }
            }
            Ok(Token::Comma) => {
                if with_input {
                    return Err("Comma not allowed after input flag".to_string());
                } else {
                    break;
                }
            }
            _ => {
                return Err(format!(
                    "{:?} is not a valid token in parse_declaration",
                    token
                ));
            }
        }
    }

    if is_declaration {
        Ok(Either::Left(Declaration {
            name: name,
            params: params,
            kind: kind,
        }))
    } else {
        dbg!(&params);
        Ok(Either::Right(Fact {
            name: name,
            params: params,
        }))
    }
}

#[derive(Debug)]
pub struct Rule {
    pub head: Declaration,
    pub body: Vec<Declaration>,
}

fn parse_rule(lexer: &mut Lexer<'_, Token>) -> Result<Rule, String> {
    let head = parse_declaration_or_fact(lexer, false, true)?
        .left()
        .unwrap();
    let mut body = vec![];

    let num_decl = match lexer.next() {
        Some(Ok(Token::Number(u))) => u,
        _ => {
            return Err("Expected a number".to_string());
        }
    };

    for _ in 0..num_decl {
        body.push(
            parse_declaration_or_fact(lexer, false, true)?
                .left()
                .unwrap(),
        );
    }

    Ok(Rule { head, body })
}

#[derive(Debug)]
pub struct Program {
    pub decls: Vec<Declaration>,
    pub rules: Vec<Rule>,
}
pub fn parse_program(lexer: &mut Lexer<'_, Token>) -> Result<Program, String> {
    let mut decls = vec![];
    let mut rules = vec![];

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::DeclHeader) => {
                decls.push(
                    parse_declaration_or_fact(lexer, true, true)?
                        .left()
                        .unwrap(),
                );
            }
            Ok(Token::RuleHeader) => {
                rules.push(parse_rule(lexer)?);
            }
            _ => {
                return Err(format!("{:?} is not a valid token", token));
            }
        }
    }

    Ok(Program { decls, rules })
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Fact {
    pub name: String,
    pub params: Vec<String>,
}
impl FactLike for Fact {
    fn name(&self) -> &str {
        &self.name
    }
    fn params(&self) -> &Vec<String> {
        &self.params
    }
}

pub fn parse_fact_vector(lexer: &mut Lexer<'_, Token>) -> Result<Vec<Fact>, String> {
    let mut facts = vec![];

    let num_facts = match lexer.next() {
        Some(Ok(Token::Number(num))) => num,
        _ => {
            return Err("Expected a number".to_string());
        }
    };

    for _ in 0..num_facts {
        facts.push(
            parse_declaration_or_fact(lexer, false, false)?
                .right()
                .unwrap(),
        );
    }

    Ok(facts)
}
