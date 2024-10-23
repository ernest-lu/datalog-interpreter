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
}

#[derive(Debug)]
pub enum DeclKind {
    Input,
    Output,
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub params: Vec<String>,
    pub kind: DeclKind,
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
fn parse_declaration(
    lexer: &mut Lexer<'_, Token>,
    with_input: bool,
) -> Result<Declaration, String> {
    // .decl <ident>(<params>) .<input/output>
    let mut name = String::new();
    let mut params = vec![];
    let mut kind = DeclKind::Input;
    dbg!(&with_input);
    while let Some(token) = lexer.next() {
        dbg!(&token);
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

    Ok(Declaration {
        name: name,
        params: params,
        kind: kind,
    })
}

#[derive(Debug)]
pub struct Rule {
    pub head: Declaration,
    pub body: Vec<Declaration>,
}

fn parse_rule(lexer: &mut Lexer<'_, Token>) -> Result<Rule, String> {
    let head = parse_declaration(lexer, false)?;
    let mut body = vec![];

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
                decls.push(parse_declaration(lexer, true)?);
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
