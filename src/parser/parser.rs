use nom::{IResult, branch::*};
use nom::bytes::complete::take;
use nom::combinator::{map, opt, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::*;
use nom::Err;
use std::result::Result::*;

use crate::ast::ast::{Expr, Ident, ImportItems, Infix, Literal, Precedence, Prefix, Program, Stmt};
use crate::lexer::token::{Token, Tokens};
use crate::parser::parser_helpers::*;

// TOKEN TAG MACRO

macro_rules! tag_token {
    ($func_name:ident, $tag:expr) => {
        #[inline]
        pub fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
            verify(take(1usize), |t: &Tokens| {
                !t.token.is_empty() && t.token[0] == $tag
            })(tokens)
        }
    };
}

// LITERAL/IDENTIFIER PARSING

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
    
    if t1.token.is_empty() {
        return Err(Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    
    // Pattern matching to know when i need to clone and when i don't
    match &t1.token[0] {
        Token::IntLiteral(n) => Ok((i1, Literal::IntLiteral(*n))),
        Token::BigIntLiteral(n) => Ok((i1, Literal::BigIntLiteral(n.clone()))),
        Token::FloatLiteral(f) => Ok((i1, Literal::FloatLitera(*f))),
        Token::StringLiteral(s) => Ok((i1, Literal::StringLiteral(s.clone()))),
        Token::BoolLiteral(b) => Ok((i1, Literal::BoolLiteral(*b))),
        Token::NullLiteral => Ok((i1, Literal::NullLiteral)),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
    }
}

fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
    let (i1, t1) = take(1usize)(input)?;
    
    if t1.token.is_empty() {
        return Err(Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    
    match &t1.token[0] {
        Token::Ident(name) => Ok((i1, Ident(name.clone()))),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
    }
}

// TOKEN TAGS

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
tag_token!(struct_tag, Token::Struct);
tag_token!(this_tag, Token::This);
tag_token!(import_tag, Token::Import);
tag_token!(while_tag, Token::While);
tag_token!(for_tag, Token::For);
tag_token!(in_tag, Token::In);
tag_token!(break_tag, Token::Break);
tag_token!(continue_tag, Token::Continue);

// OPERATOR PRECEDENCE

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
        Token::Modulo => (Precedence::PProduct, Some(Infix::Modulo)),
        Token::LParen => (Precedence::PCall, None),
        Token::LBracket => (Precedence::PIndex, None),
        Token::Dot => (Precedence::PCall, None),
        _ => (Precedence::PLowest, None),
    }
}

// PROGRAM AND STATEMENTS

fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
    terminated(many0(parse_stmt), eof_tag)(input)
}

fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    parse_pratt_expr(input, Precedence::PLowest)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    alt((
        parse_import_stmt,
        parse_let_stmt,
        parse_fn_stmt,
        parse_return_stmt,
        parse_struct_stmt,
        parse_while_stmt,
        parse_for_stmt,
        parse_break_stmt,
        parse_continue_stmt,
        parse_assign_or_expr_stmt,
    ))(input)
}

// ASSIGNMENT/EXPRESSION PARSING

fn parse_assign_or_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    // Fast path: simple identifier assignment
    if matches!(peek_token(input), Some(Token::Ident(_))) {
        if let Ok((after_ident, ident)) = parse_ident(input) {
            if peek_matches(after_ident, Token::Assign) {
                let (i1, _) = assign_tag(after_ident)?;
                let (i2, expr) = parse_expr(i1)?;
                let (i3, _) = semicolon_tag(i2)?;
                return Ok((i3, Stmt::AssignStmt(ident, expr)));
            }
        }
    }
    
    // Try complex assignments
    if let Ok((after_lhs, lhs)) = parse_atom_expr(input) {
        match peek_token(after_lhs) {
            Some(Token::Dot) => {
                if let Ok(res) = try_parse_field_assignment(after_lhs, lhs.clone()) {
                    return Ok(res);
                }
            }
            Some(Token::LBracket) => {
                if let Ok(res) = try_parse_index_assignment(after_lhs, lhs.clone()) {
                    return Ok(res);
                }
            }
            _ => {}
        }
    }
    
    // Fallback
    parse_expr_stmt(input)
}

fn try_parse_field_assignment(input: Tokens, object: Expr) -> IResult<Tokens, Stmt> {
    let (i1, _) = dot_tag(input)?;
    let (i2, Ident(field_name)) = parse_ident(i1)?;
    
    if !peek_matches(i2, Token::Assign) {
        return Err(Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    
    let (i3, _) = assign_tag(i2)?;
    let (i4, value) = parse_expr(i3)?;
    let (i5, _) = semicolon_tag(i4)?;
    
    Ok((i5, Stmt::FieldAssignStmt {
        object: Box::new(object),
        field: field_name,
        value: Box::new(value),
    }))
}

fn try_parse_index_assignment(input: Tokens, target: Expr) -> IResult<Tokens, Stmt> {
    let (i1, _) = lbracket_tag(input)?;
    let (i2, index) = parse_expr(i1)?;
    let (i3, _) = rbracket_tag(i2)?;
    
    if !peek_matches(i3, Token::Assign) {
        return Err(Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    
    let (i4, _) = assign_tag(i3)?;
    let (i5, value) = parse_expr(i4)?;
    let (i6, _) = semicolon_tag(i5)?;
    
    Ok((i6, Stmt::IndexAssignStmt {
        target: Box::new(target),
        index: Box::new(index),
        value: Box::new(value),
    }))
}

// STATEMENT PARSERS

fn parse_let_stmt_no_semicolon(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((let_tag, parse_ident, assign_tag, parse_expr)),
        |(_, ident, _, expr)| Stmt::LetStmt(ident, expr),
    )(input)
}

fn parse_assign_stmt_no_semicolon(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((parse_ident, assign_tag, parse_expr)),
        |(ident, _, expr)| Stmt::AssignStmt(ident, expr),
    )(input)
}

fn parse_break_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(terminated(break_tag, semicolon_tag), |_| Stmt::BreakStmt)(input)
}

fn parse_continue_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(terminated(continue_tag, semicolon_tag), |_| Stmt::ContinueStmt)(input)
}

fn parse_let_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((let_tag, parse_ident, assign_tag, parse_expr, semicolon_tag)),
        |(_, ident, _, expr, _)| Stmt::LetStmt(ident, expr),
    )(input)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((return_tag, parse_expr, semicolon_tag)),
        |(_, expr, _)| Stmt::ReturnStmt(expr),
    )(input)
}

fn parse_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (after_expr, expr) = parse_expr(input)?;
    
    if peek_matches(after_expr, Token::SemiColon) {
        let (i1, _) = semicolon_tag(after_expr)?;
        Ok((i1, Stmt::ExprStmt(expr)))
    } else {
        Ok((after_expr, Stmt::ExprValueStmt(expr)))
    }
}

fn parse_block_stmt(input: Tokens) -> IResult<Tokens, Program> {
    braced(many0(parse_stmt))(input)
}

fn parse_fn_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            function_tag,
            parse_ident,
            parens(comma_separated0(parse_ident)),
            parse_block_stmt,
        )),
        |(_, name, params, body)| Stmt::FnStmt { name, params, body },
    )(input)
}

// EXPRESSION PARSING

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_literal_expr,
        parse_ident_or_struct_literal,
        parse_fn_expr,
        parse_if_expr,
        parse_this_expr,
        parse_array_expr,
        parse_hash_expr,
        parse_prefix_expr,
        parens(parse_expr),
    ))(input)
}

fn parse_pratt_expr(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (mut i, mut left) = parse_atom_expr(input)?;

    loop {
        let Some(curr_token) = peek_token(i) else { break };
        let (peek_precedence, _) = infix_op(curr_token);

        if precedence >= peek_precedence || peek_precedence == Precedence::PLowest {
            break;
        }

        match curr_token {
            Token::LParen => {
                let (i2, args) = parens(comma_separated0(parse_expr))(i)?;
                left = Expr::CallExpr {
                    function: Box::new(left),
                    arguments: args,
                };
                i = i2;
            }
            Token::LBracket => {
                let (i2, index) = bracketed(parse_expr)(i)?;
                left = Expr::IndexExpr {
                    array: Box::new(left),
                    index: Box::new(index),
                };
                i = i2;
            }
            Token::Dot => {
                let (i1, _) = dot_tag(i)?;
                let (i2, Ident(field_name)) = parse_ident(i1)?;

                if peek_matches(i2, Token::LParen) {
                    let (i3, args) = parens(comma_separated0(parse_expr))(i2)?;
                    left = Expr::MethodCallExpr {
                        object: Box::new(left),
                        method: field_name,
                        arguments: args,
                    };
                    i = i3;
                } else {
                    left = Expr::FieldAccessExpr {
                        object: Box::new(left),
                        field: field_name,
                    };
                    i = i2;
                }
            }
            _ => {
                let (_, infix_op_opt) = infix_op(curr_token);
                if let Some(infix) = infix_op_opt {
                    let (i1, _) = take(1usize)(i)?;
                    let (i2, right) = parse_pratt_expr(i1, peek_precedence)?;
                    left = Expr::InfixExpr(infix, Box::new(left), Box::new(right));
                    i = i2;
                } else {
                    break;
                }
            }
        }
    }

    Ok((i, left))
}

fn parse_literal_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::LitExpr)(input)
}

fn parse_ident_or_struct_literal(input: Tokens) -> IResult<Tokens, Expr> {
    let (after_ident, ident) = parse_ident(input)?;
    
    if peek_matches(after_ident, Token::LBrace) {
        let (i1, fields) = braced(comma_separated0(separated_pair(
            parse_ident,
            colon_tag,
            parse_expr,
        )))(after_ident)?;
        Ok((i1, Expr::StructLiteral { name: ident, fields }))
    } else {
        Ok((after_ident, Expr::IdentExpr(ident)))
    }
}

fn parse_prefix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;
    
    if t1.token.is_empty() {
        return Err(Err::Error(Error::new(input, ErrorKind::Tag)));
    }

    let prefix = match t1.token[0] {
        Token::Not => Prefix::Not,
        Token::Plus => Prefix::PrefixPlus,
        Token::Minus => Prefix::PrefixMinus,
        _ => return Err(Err::Error(Error::new(input, ErrorKind::Tag))),
    };

    let (i2, expr) = parse_pratt_expr(i1, Precedence::PPrefix)?;
    Ok((i2, Expr::PrefixExpr(prefix, Box::new(expr))))
}

fn parse_if_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            if_tag,
            parens(parse_expr),
            parse_block_stmt,
            opt(preceded(else_tag, parse_block_stmt)),
        )),
        |(_, cond, consequence, alternative)| Expr::IfExpr {
            cond: Box::new(cond),
            consequence,
            alternative,
        },
    )(input)
}

fn parse_fn_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((function_tag, parens(comma_separated0(parse_ident)), parse_block_stmt)),
        |(_, params, body)| Expr::FnExpr { params, body },
    )(input)
}

fn parse_array_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(bracketed(comma_separated0(parse_expr)), Expr::ArrayExpr)(input)
}

fn parse_hash_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        braced(comma_separated0(separated_pair(parse_expr, colon_tag, parse_expr))),
        Expr::HashExpr,
    )(input)
}

fn parse_this_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(this_tag, |_| Expr::ThisExpr)(input)
}

// STRUCT PARSING

fn parse_struct_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            struct_tag,
            parse_ident,
            braced(comma_separated0(separated_pair(parse_ident, colon_tag, parse_expr))),
        )),
        |(_, name, pairs)| {
            let mut fields = Vec::new();
            let mut methods = Vec::new();
            
            for (ident, expr) in pairs {
                match expr {
                    Expr::FnExpr { .. } => methods.push((ident, expr)),
                    _ => fields.push((ident, expr)),
                }
            }
            
            Stmt::StructStmt { name, fields, methods }
        },
    )(input)
}

// LOOP STATEMENTS

fn parse_while_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((while_tag, parens(parse_expr), parse_block_stmt)),
        |(_, cond, body)| Stmt::ExprStmt(Expr::WhileExpr {
            cond: Box::new(cond),
            body,
        }),
    )(input)
}

fn parse_for_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, _) = for_tag(input)?;
    let (i2, _) = lparen_tag(i1)?;

    match peek_token(i2) {
        Some(Token::Let) => parse_c_style_for(i2),
        Some(Token::Ident(_)) => {
            if let Ok((after_ident, _)) = parse_ident(i2) {
                if peek_matches(after_ident, Token::In) {
                    parse_for_in_loop(i2)
                } else {
                    parse_c_style_for(i2)
                }
            } else {
                parse_c_style_for(i2)
            }
        }
        _ => parse_c_style_for(i2),
    }
}

fn parse_for_in_loop(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, ident) = parse_ident(input)?;
    let (i2, _) = in_tag(i1)?;
    let (i3, iterable) = parse_expr(i2)?;
    let (i4, _) = rparen_tag(i3)?;
    let (i5, body) = parse_block_stmt(i4)?;
    
    Ok((i5, Stmt::ExprStmt(Expr::ForExpr {
        ident,
        iterable: Box::new(iterable),
        body,
    })))
}

fn parse_c_style_for(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, init) = opt(alt((
        map(parse_let_stmt_no_semicolon, Box::new),
        map(parse_assign_stmt_no_semicolon, Box::new),
    )))(input)?;
    
    let (i2, _) = semicolon_tag(i1)?;
    let (i3, cond) = opt(map(parse_expr, Box::new))(i2)?;
    let (i4, _) = semicolon_tag(i3)?;
    let (i5, update) = opt(map(parse_assign_stmt_no_semicolon, Box::new))(i4)?;
    let (i6, _) = rparen_tag(i5)?;
    let (i7, body) = parse_block_stmt(i6)?;
    
    Ok((i7, Stmt::ExprStmt(Expr::CStyleForExpr {
        init,
        cond,
        update,
        body,
    })))
}

// IMPORT STATEMENT

fn parse_import_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, _) = import_tag(input)?;
    let (i2, Ident(first)) = parse_ident(i1)?;
    let mut path = vec![first];

    let (i3, rest) = many0(preceded(dot_tag, parse_ident))(i2)?;
    for Ident(name) in rest {
        path.push(name);
    }
    
    let (i4, items) = if peek_matches(i3, Token::Dot) {
        let (i_dot, _) = dot_tag(i3)?;
        let (i_items, idents) = braced(comma_separated1(parse_ident))(i_dot)?;
        let names = idents.into_iter().map(|Ident(n)| n).collect();
        (i_items, ImportItems::Specific(names))
    } else {
        (i3, ImportItems::All)
    };
    
    let (i5, _) = semicolon_tag(i4)?;
    Ok((i5, Stmt::ImportStmt { path, items }))
}

pub struct Parser;

impl Parser {
    pub fn parse_tokens(tokens: Tokens) -> IResult<Tokens, Program> {
        parse_program(tokens)
    }
}