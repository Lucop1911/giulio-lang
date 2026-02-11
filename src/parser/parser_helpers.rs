use crate::lexer::token::{Token, Tokens};
use crate::parser::parser::*;
use nom::{IResult, branch::alt, multi::many0};
use nom::sequence::*;
use std::result::Result::*;

// Peek at the next token without consuming it
#[inline]
pub fn peek_token(input: Tokens<'_>) -> Option<&'_ Token> {
    input.token.first()
}

// Check if next token matches expected
#[inline]
pub fn peek_matches(input: Tokens, expected: Token) -> bool {
    peek_token(input).map_or(false, |t| *t == expected)
}

// Parse comma-separated items (at least one)
pub fn comma_separated1<'a, F, O>(
    mut item_parser: F,
) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, Vec<O>>
where
    F: FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>,
{
    move |input| {
        let (i1, first) = item_parser(input)?;
        let (i2, rest) = many0(preceded(comma_tag, &mut item_parser))(i1)?;
        
        let mut result = Vec::with_capacity(1 + rest.len());
        result.push(first);
        result.extend(rest);
        Ok((i2, result))
    }
}

// Parse comma-separated items (empty allowed)
pub fn comma_separated0<'a, F, O>(
    item_parser: F,
) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, Vec<O>>
where
    F: FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>,
{
    alt((comma_separated1(item_parser), |input| Ok((input, vec![]))))
}

// Wrap parser in braces { }
pub fn braced<'a, F, O>(parser: F) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>
where
    F: FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>,
{
    delimited(lbrace_tag, parser, rbrace_tag)
}

// Wrap parser in parentheses ( )
pub fn parens<'a, F, O>(parser: F) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>
where
    F: FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>,
{
    delimited(lparen_tag, parser, rparen_tag)
}

// Wrap parser in brackets [ ]
pub fn bracketed<'a, F, O>(parser: F) -> impl FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>
where
    F: FnMut(Tokens<'a>) -> IResult<Tokens<'a>, O>,
{
    delimited(lbracket_tag, parser, rbracket_tag)
}