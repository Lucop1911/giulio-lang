use nom::{IResult, branch::*, error_position};
use nom::bytes::complete::take;
use nom::combinator::{map, opt, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::*;
use nom::Err;
use std::result::Result::*;

use crate::ast::ast::{Expr, Ident, ImportItems, Infix, Literal, Precedence, Prefix, Program, Stmt};
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
            Token::BigIntLiteral(name) => Ok((i1, Literal::BigIntLiteral(name))),
            Token::FloatLiteral(name) => Ok((i1, Literal::FloatLitera(name))),
            Token::StringLiteral(s) => Ok((i1, Literal::StringLiteral(s))),
            Token::BoolLiteral(b) => Ok((i1, Literal::BoolLiteral(b))),
            Token::NullLiteral => Ok((i1, Literal::NullLiteral)),
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
tag_token!(struct_tag, Token::Struct);
tag_token!(this_tag, Token::This);
tag_token!(import_tag, Token::Import);
tag_token!(while_tag, Token::While);
tag_token!(for_tag, Token::For);
tag_token!(in_tag, Token::In);
tag_token!(break_tag, Token::Break);
tag_token!(continue_tag, Token::Continue);

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

// Helper to parse let statement without semicolon
fn parse_let_stmt_no_semicolon(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            let_tag,
            parse_ident,
            assign_tag,
            parse_expr,
        )),
        |(_, ident, _, expr)| Stmt::LetStmt(ident, expr),
    )(input)
}

// Helper to parse assignment without semicolon
fn parse_assign_stmt_no_semicolon(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            parse_ident,
            assign_tag,
            parse_expr,
        )),
        |(ident, _, expr)| Stmt::AssignStmt(ident, expr),
    )(input)
}

fn parse_break_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        terminated(break_tag, semicolon_tag),
        |_| Stmt::BreakStmt,
    )(input)
}

fn parse_continue_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        terminated(continue_tag, semicolon_tag),
        |_| Stmt::ContinueStmt,
    )(input)
}

fn parse_assign_or_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    // Try to parse as identifier assignment: ident = expr
    if let Ok((after_ident, ident)) = parse_ident(input) 
        && let Ok((_, next_tokens)) = take::<_, _, Error<_>>(1usize)(after_ident) {
            if !next_tokens.token.is_empty() && next_tokens.token[0] == Token::Assign {
                let (i1, _) = assign_tag(after_ident)?;
                let (i2, expr) = parse_expr(i1)?;
                let (i3, _) = (semicolon_tag)(i2)?;
                return Ok((i3, Stmt::AssignStmt(ident, expr)));
            }
    }
    
    // Attempt parsing as field assignment: expr.field = expr
    // This handles: this.(ident) = value, obj.(field) = value, etc.
    if let Ok((after_expr, object_expr)) = parse_atom_expr(input) 
        && let Ok((_, next_token)) = take::<_, _, Error<_>>(1usize)(after_expr) {
            if !next_token.token.is_empty() && next_token.token[0] == Token::Dot {
                // We got object.something - check if it's a field assignment
                let (after_dot, _) = dot_tag(after_expr)?;
                if let Ok((after_field, field_ident)) = parse_ident(after_dot) {
                    let Ident(field_name) = field_ident;
                    // Check if next token is =
                    if let Ok((_, assign_token)) = take::<_, _, Error<_>>(1usize)(after_field) {
                        if !assign_token.token.is_empty() && assign_token.token[0] == Token::Assign {
                            // It's a field assignment!
                            let (i1, _) = assign_tag(after_field)?;
                            let (i2, value_expr) = parse_expr(i1)?;
                            let (i3, _) = semicolon_tag(i2)?;
                            return Ok((i3, Stmt::FieldAssignStmt {
                                object: Box::new(object_expr),
                                field: field_name,
                                value: Box::new(value_expr),
                            }));
                        }
                    }
                }
            }
    }
    
    // Fallback to expression statement
    parse_expr_stmt(input)
}

fn parse_let_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            let_tag,
            parse_ident,
            assign_tag,
            parse_expr,
            (semicolon_tag),
        )),
        |(_, ident, _, expr, _)| Stmt::LetStmt(ident, expr),
    )(input)
}

fn parse_fn_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            function_tag,
            parse_ident,
            lparen_tag,
            alt((parse_params, empty_params)),
            rparen_tag,
            parse_block_stmt,
            opt(semicolon_tag)
        )),
        |(_, name, _, params, _, body, _)| Stmt::FnStmt {
            name,
            params,
            body,
        },
    )(input)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        delimited(return_tag, parse_expr, opt(semicolon_tag)),
        Stmt::ReturnStmt,
    )(input)
}

fn parse_expr_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, expr) = parse_expr(input)?;
    
    // Check if this expression ends with a block (if, while, for, fn)
    // If so, semicolon is optional
    let needs_semicolon = !matches!(
        expr,
        Expr::IfExpr { .. } | Expr::WhileExpr { .. } | Expr::ForExpr { .. } | Expr::CStyleForExpr { .. } | Expr::FnExpr { .. }
    );
    
    if needs_semicolon {
        let (i2, _) = semicolon_tag(i1)?;
        Ok((i2, Stmt::ExprStmt(expr)))
    } else {
        let (i2, _) = opt(semicolon_tag)(i1)?;
        Ok((i2, Stmt::ExprStmt(expr)))
    }
}

fn parse_block_stmt(input: Tokens) -> IResult<Tokens, Program> {
    delimited(lbrace_tag, many0(parse_stmt), rbrace_tag)(input)
}

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_lit_expr,
        parse_this_expr,
        parse_struct_literal,
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
    opt(preceded(
        else_tag,
        alt((
            parse_block_stmt,
            map(parse_if_expr, |expr| vec![Stmt::ExprStmt(expr)]),
        )),
    ))(input)
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
        // It's a method call
        map(
            delimited(lparen_tag, alt((parse_exprs, empty_boxed_vec)), rparen_tag),
            |args| Expr::MethodCallExpr {
                object: Box::new(object.clone()),
                method: method_name.clone(),
                arguments: args,
            },
        )(i2)
    } else {
        // It's a field access
        Ok((i2, Expr::FieldAccessExpr {
            object: Box::new(object),
            field: method_name,
        }))
    }
}

fn parse_struct_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            struct_tag,
            parse_ident,
            lbrace_tag,
            parse_struct_body,
            rbrace_tag,
            opt(semicolon_tag)
        )),
        |(_, name, _, (fields, methods), _, _)| Stmt::StructStmt {
            name,
            fields,
            methods,
        },
    )(input)
}

fn parse_struct_body(input: Tokens) -> IResult<Tokens, (Vec<(Ident, Expr)>, Vec<(Ident, Expr)>)> {
    let (i1, pairs) = alt((parse_struct_pairs, empty_struct_pairs))(input)?;
    
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    
    for (ident, expr) in pairs {
        match expr {
            Expr::FnExpr { .. } => methods.push((ident, expr)),
            _ => fields.push((ident, expr)),
        }
    }
    Ok((i1, (fields, methods)))
}

fn parse_struct_pair(input: Tokens) -> IResult<Tokens, (Ident, Expr)> {
    separated_pair(parse_ident, colon_tag, parse_expr)(input)
}

fn parse_struct_comma_pair(input: Tokens) -> IResult<Tokens, (Ident, Expr)> {
    preceded(comma_tag, parse_struct_pair)(input)
}

fn parse_struct_pairs(input: Tokens) -> IResult<Tokens, Vec<(Ident, Expr)>> {
    map(
        pair(parse_struct_pair, many0(parse_struct_comma_pair)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}

fn empty_struct_pairs(input: Tokens) -> IResult<Tokens, Vec<(Ident, Expr)>> {
    Ok((input, vec![]))
}

fn parse_struct_literal(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            parse_ident,
            lbrace_tag,
            alt((parse_field_assignments, empty_field_assignments)),
            rbrace_tag,
        )),
        |(name, _, fields, _)| Expr::StructLiteral { name, fields },
    )(input)
}

fn parse_field_assignment(input: Tokens) -> IResult<Tokens, (Ident, Expr)> {
    separated_pair(parse_ident, colon_tag, parse_expr)(input)
}

fn parse_comma_field_assignment(input: Tokens) -> IResult<Tokens, (Ident, Expr)> {
    preceded(comma_tag, parse_field_assignment)(input)
}

fn parse_field_assignments(input: Tokens) -> IResult<Tokens, Vec<(Ident, Expr)>> {
    map(
        pair(parse_field_assignment, many0(parse_comma_field_assignment)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}

fn empty_field_assignments(input: Tokens) -> IResult<Tokens, Vec<(Ident, Expr)>> {
    Ok((input, vec![]))
}

fn parse_this_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(this_tag, |_| Expr::ThisExpr)(input)
}

fn parse_while_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            while_tag,
            lparen_tag,
            parse_expr,
            rparen_tag,
            parse_block_stmt,
        )),
        |(_, _, cond, _, body)| Stmt::ExprStmt(Expr::WhileExpr {
            cond: Box::new(cond),
            body,
        }),
    )(input)
}

fn parse_for_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, _) = for_tag(input)?;
    let (i2, _) = lparen_tag(i1)?;

    // Has "let" -> c style
    // Ident followed by "in" -> for-in
    // Ident followed by "=" -> c style
    
    let (_lookahead, first_token) = take(1usize)(i2)?;
    
    if !first_token.token.is_empty() {
        match &first_token.token[0] {
            Token::Let => {
                return parse_c_style_for(i2);
            }
            Token::Ident(_) => {
                if let Ok((after_ident, _)) = parse_ident(i2) {
                    if let Ok((_, next_token)) = take::<_, _, nom::error::Error<_>>(1usize)(after_ident) {
                        if !next_token.token.is_empty() {
                            match &next_token.token[0] {
                                Token::In => {
                                    // for-in loop: for (fruit in fruits)
                                    // wont use parse_c_style_for, parse directly
                                }
                                Token::Assign => {
                                    // c style loop
                                    return parse_c_style_for(i2);
                                }
                                _ => {
                                    // Unknown, try for-in as fallback
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Something else, probably an error but try for-in
            }
        }
    }
    
    // Parse as for-in loop
    let (i3, ident) = parse_ident(i2)?;
    let (i4, _) = in_tag(i3)?;
    let (i5, iterable) = parse_expr(i4)?;
    let (i6, _) = rparen_tag(i5)?;
    let (i7, body) = parse_block_stmt(i6)?;
    
    Ok((i7, Stmt::ExprStmt(Expr::ForExpr {
        ident,
        iterable: Box::new(iterable),
        body,
    })))
}

fn parse_c_style_for(input: Tokens) -> IResult<Tokens, Stmt> {
    // Parse init (optional let statement or assignment)
    let (i1, init) = opt(alt((
        map(parse_let_stmt_no_semicolon, |stmt| Box::new(stmt)),
        map(parse_assign_stmt_no_semicolon, |stmt| Box::new(stmt)),
    )))(input)?;
    
    let (i2, _) = semicolon_tag(i1)?;
    
    // Parse condition (optional expression)
    let (i3, cond) = opt(map(parse_expr, |expr| Box::new(expr)))(i2)?;
    
    let (i4, _) = semicolon_tag(i3)?;
    
    // Parse update (optional assignment)
    let (i5, update) = opt(map(parse_assign_stmt_no_semicolon, |stmt| Box::new(stmt)))(i4)?;
    
    let (i6, _) = rparen_tag(i5)?;
    let (i7, body) = parse_block_stmt(i6)?;
    
    Ok((i7, Stmt::ExprStmt(Expr::CStyleForExpr {
        init,
        cond,
        update,
        body,
    })))
}

fn parse_import_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    let (i1, _) = import_tag(input)?;

    let (i2, first_ident) = parse_ident(i1)?;
    let Ident(first) = first_ident;
    let mut path = vec![first];

    let (i3, rest) = many0(preceded(dot_tag, parse_ident))(i2)?;
    for Ident(name) in rest {
        path.push(name);
    }
    
    // Check what comes next
    let (_i4, t4) = take(1usize)(i3)?;
    
    let (i5, items) = if !t4.token.is_empty() && t4.token[0] == Token::Dot {
        // We have .{...} for specific imports
        let (i_dot, _) = dot_tag(i3)?;
        let (i_items, parsed_items) = parse_specific_imports_body(i_dot)?;
        (i_items, parsed_items)
    } else {
        // Import everything
        (i3, ImportItems::All)
    };
    
    let (i6, _) = semicolon_tag(i5)?;
    
    Ok((i6, Stmt::ImportStmt { path, items }))
}

fn parse_specific_imports_body(input: Tokens) -> IResult<Tokens, ImportItems> {
    delimited(
        lbrace_tag,
        map(
            pair(
                parse_ident,
                many0(preceded(comma_tag, parse_ident))
            ),
            |(first, rest)| {
                let Ident(first_name) = first;
                let mut items = vec![first_name];
                for Ident(name) in rest {
                    items.push(name);
                }
                ImportItems::Specific(items)
            }
        ),
        rbrace_tag,
    )(input)
}

pub struct Parser;

impl Parser {
    pub fn parse_tokens(tokens: Tokens) -> IResult<Tokens, Program> {
        parse_program(tokens)
    }
}