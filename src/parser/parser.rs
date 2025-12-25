use nom::{IResult, branch::*, error_position};
use nom::bytes::complete::take;
use nom::combinator::{map, opt, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::*;
use nom::Err;
use std::result::Result::*;

use crate::ast::ast::{Expr, Ident, Infix, Literal, Precedence, Prefix, Program, Stmt};
use crate::lexer::token::{Token, Tokens};

macro_rules! tag_token (
    ($func_name:ident, $tag: expr) => (
        fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
            verify(take(1usize), |t: &Tokens| t.token[0] == $tag)(tokens)
        }
    )
  );

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.token.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t1.token[0].clone() {
            Token::IntLiteral(name) => Ok((i1, Literal::IntLiteral(name))),
            Token::StringLiteral(s) => Ok((i1, Literal::StringLiteral(s))),
            Token::BoolLiteral(b) => Ok((i1, Literal::BoolLiteral(b))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.token.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t1.token[0].clone() {
            Token::Ident(name) => Ok((i1, Ident(name))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

tag_token!(let_tag, Token::Let);
tag_token!(assign_tag, Token::Assign);
tag_token!(semicolon_tag, Token::SemiColon);
tag_token!(return_tag, Token::Return);
tag_token!(lbrace_tag, Token::LBrace);
tag_token!(rbrace_tag, Token::RBrace);
tag_token!(lparen_tag, Token::LParen);
tag_token!(rparen_tag, Token::RParen);
tag_token!(lbracket_tag, Token::LBracket);
tag_token!(rbracket_tag, Token::RBracket);
tag_token!(comma_tag, Token::Comma);
tag_token!(colon_tag, Token::Colon);
tag_token!(plus_tag, Token::Plus);
tag_token!(minus_tag, Token::Minus);
tag_token!(not_tag, Token::Not);
tag_token!(if_tag, Token::If);
tag_token!(else_tag, Token::Else);
tag_token!(function_tag, Token::Function);
tag_token!(eof_tag, Token::EOF);
tag_token!(dot_tag, Token::Dot);

fn infix_op(t: &Token) -> (Precedence, Option<Infix>) {
    match *t {
        Token::Or => (Precedence::POr, Some(Infix::Or)),
        Token::And => (Precedence::PAnd, Some(Infix::And)),
        Token::Equal => (Precedence::PEquals, Some(Infix::Equal)),
        Token::NotEqual => (Precedence::PEquals, Some(Infix::NotEqual)),
        Token::LessThanEqual => (Precedence::PLessGreater, Some(Infix::LessThanEqual)),
        Token::GreaterThanEqual => (Precedence::PLessGreater, Some(Infix::GreaterThanEqual)),
        Token::LessThan => (Precedence::PLessGreater, Some(Infix::LessThan)),
        Token::GreaterThan => (Precedence::PLessGreater, Some(Infix::GreaterThan)),
        Token::Plus => (Precedence::PSum, Some(Infix::Plus)),
        Token::Minus => (Precedence::PSum, Some(Infix::Minus)),
        Token::Multiply => (Precedence::PProduct, Some(Infix::Multiply)),
        Token::Divide => (Precedence::PProduct, Some(Infix::Divide)),
        Token::LParen => (Precedence::PCall, None),
        Token::LBracket => (Precedence::PIndex, None),
        Token::Dot => (Precedence::PCall, None),
        _ => (Precedence::PLowest, None),
    }
}

fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
    terminated(many0(parse_stmt), eof_tag)(input)
}

fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    parse_pratt_expr(input, Precedence::PLowest)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    alt((parse_let_stmt, parse_return_stmt, parse_expr_stmt))(input)
}

fn parse_let_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            let_tag,
            parse_ident,
            assign_tag,
            parse_expr,
            opt(semicolon_tag),
        )),
        |(_, ident, _, expr, _)| Stmt::LetStmt(ident, expr),
    )(input)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        delimited(return_tag, parse_expr, opt(semicolon_tag)),
        Stmt::ReturnStmt,
    )(input)
}

fn parse_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(terminated(parse_expr, opt(semicolon_tag)), |expr| {
        Stmt::ExprStmt(expr)
    })(input)
}

fn parse_block_stmt(input: Tokens) -> IResult<Tokens, Program> {
    delimited(lbrace_tag, many0(parse_stmt), rbrace_tag)(input)
}

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_lit_expr,
        parse_ident_expr,
        parse_prefix_expr,
        parse_paren_expr,
        parse_array_expr,
        parse_hash_expr,
        parse_if_expr,
        parse_fn_expr,
    ))(input)
}

fn parse_paren_expr(input: Tokens) -> IResult<Tokens, Expr> {
    delimited(lparen_tag, parse_expr, rparen_tag)(input)
}

fn parse_lit_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::LitExpr)(input)
}

fn parse_ident_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_ident, Expr::IdentExpr)(input)
}

fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
    preceded(comma_tag, parse_expr)(input)
}

fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    map(
        pair(parse_expr, many0(parse_comma_exprs)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}

fn empty_boxed_vec(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    Ok((input, vec![]))
}

fn parse_array_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        delimited(
            lbracket_tag,
            alt((parse_exprs, empty_boxed_vec)),
            rbracket_tag,
        ),
        Expr::ArrayExpr,
    )(input)
}

fn parse_hash_pair(input: Tokens) -> IResult<Tokens, (Expr, Expr)> {
    separated_pair(parse_expr, colon_tag, parse_expr)(input)
}

fn parse_hash_comma_expr(input: Tokens) -> IResult<Tokens, (Expr, Expr)> {
    preceded(comma_tag, parse_hash_pair)(input)
}

fn parse_hash_pairs(input: Tokens) -> IResult<Tokens, Vec<(Expr, Expr)>> {
    map(
        pair(parse_hash_pair, many0(parse_hash_comma_expr)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}
fn empty_pairs(input: Tokens) -> IResult<Tokens, Vec<(Expr, Expr)>> {
    Ok((input, vec![]))
}

fn parse_hash_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        delimited(lbrace_tag, alt((parse_hash_pairs, empty_pairs)), rbrace_tag),
        Expr::HashExpr,
    )(input)
}

fn parse_prefix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    let (i1, t1) = alt((plus_tag, minus_tag, not_tag))(input)?;
    if t1.token.is_empty() {
        Err(Err::Error(error_position!(input, ErrorKind::Tag)))
    } else {
        let (i2, e) = parse_atom_expr(i1)?;
        match t1.token[0].clone() {
            Token::Plus => Ok((i2, Expr::PrefixExpr(Prefix::PrefixPlus, Box::new(e)))),
            Token::Minus => Ok((i2, Expr::PrefixExpr(Prefix::PrefixMinus, Box::new(e)))),
            Token::Not => Ok((i2, Expr::PrefixExpr(Prefix::Not, Box::new(e)))),
            _ => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
        }
    }
}

fn parse_pratt_expr(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, left) = parse_atom_expr(input)?;
    go_parse_pratt_expr(i1, precedence, left)
}

fn go_parse_pratt_expr(input: Tokens, precedence: Precedence, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;

    if t1.token.is_empty() {
        Ok((i1, left))
    } else {
        let preview = &t1.token[0];
        let p = infix_op(preview);
        match p {
            (Precedence::PCall, _) if precedence < Precedence::PCall => {
                match preview {
                    Token::LParen => {
                        let (i2, left2) = parse_call_expr(input, left)?;
                        go_parse_pratt_expr(i2, precedence, left2)
                    }
                    Token::Dot => {
                        let (i2, left2) = parse_method_call_expr(input, left)?;
                        go_parse_pratt_expr(i2, precedence, left2)
                    }
                    _ => Ok((input, left))
                }
            }
            (Precedence::PIndex, _) if precedence < Precedence::PIndex => {
                let (i2, left2) = parse_index_expr(input, left)?;
                go_parse_pratt_expr(i2, precedence, left2)
            }
            (ref peek_precedence, _) if precedence < *peek_precedence => {
                let (i2, left2) = parse_infix_expr(input, left)?;
                go_parse_pratt_expr(i2, precedence, left2)
            }
            _ => Ok((input, left)),
        }
    }
}

fn parse_infix_expr(input: Tokens, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.token.is_empty() {
        Err(Err::Error(error_position!(input, ErrorKind::Tag)))
    } else {
        let next = &t1.token[0];
        let (precedence, maybe_op) = infix_op(next);
        match maybe_op {
            None => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
            Some(op) => {
                let (i2, right) = parse_pratt_expr(i1, precedence)?;
                Ok((i2, Expr::InfixExpr(op, Box::new(left), Box::new(right))))
            }
        }
    }
}

fn parse_call_expr(input: Tokens, fn_handle: Expr) -> IResult<Tokens, Expr> {
    map(
        delimited(lparen_tag, alt((parse_exprs, empty_boxed_vec)), rparen_tag),
        |e| Expr::CallExpr {
            function: Box::new(fn_handle.clone()),
            arguments: e,
        },
    )(input)
}

fn parse_index_expr(input: Tokens, arr: Expr) -> IResult<Tokens, Expr> {
    map(delimited(lbracket_tag, parse_expr, rbracket_tag), |idx| {
        Expr::IndexExpr {
            array: Box::new(arr.clone()),
            index: Box::new(idx),
        }
    })(input)
}

fn parse_if_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            if_tag,
            lparen_tag,
            parse_expr,
            rparen_tag,
            parse_block_stmt,
            parse_else_expr,
        )),
        |(_, _, expr, _, c, a)| Expr::IfExpr {
            cond: Box::new(expr),
            consequence: c,
            alternative: a,
        },
    )(input)
}

fn parse_else_expr(input: Tokens) -> IResult<Tokens, Option<Program>> {
    opt(preceded(else_tag, parse_block_stmt))(input)
}

fn empty_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> {
    Ok((input, vec![]))
}

fn parse_fn_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            function_tag,
            lparen_tag,
            alt((parse_params, empty_params)),
            rparen_tag,
            parse_block_stmt,
        )),
        |(_, _, p, _, b)| Expr::FnExpr { params: p, body: b },
    )(input)
}

fn parse_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> {
    map(
        pair(parse_ident, many0(preceded(comma_tag, parse_ident))),
        |(p, ps)| [&vec![p][..], &ps[..]].concat(),
    )(input)
}

fn parse_method_call_expr(input: Tokens, object: Expr) -> IResult<Tokens, Expr> {
    let (i1, _) = dot_tag(input)?;
    let (i2, method_ident) = parse_ident(i1)?;
    let Ident(method_name) = method_ident;
    
    let (_i3, t3) = take(1usize)(i2)?;
    if !t3.token.is_empty() && t3.token[0] == Token::LParen {
        map(
            delimited(lparen_tag, alt((parse_exprs, empty_boxed_vec)), rparen_tag),
            |args| Expr::MethodCallExpr {
                object: Box::new(object.clone()),
                method: method_name.clone(),
                arguments: args,
            },
        )(i2)
    } else {
        Ok((i2, Expr::MethodCallExpr {
            object: Box::new(object),
            method: method_name,
            arguments: vec![],
        }))
    }
}

pub struct Parser;

impl Parser {
    pub fn parse_tokens(tokens: Tokens) -> IResult<Tokens, Program> {
        parse_program(tokens)
    }
}