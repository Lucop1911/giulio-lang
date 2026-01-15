use nom::branch::*;
use nom::bytes::complete::{tag, take, is_not};
use nom::character::complete::{alpha1, alphanumeric1, digit1, multispace0, line_ending};
use nom::combinator::{map, map_res, recognize, value, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded};
use nom::*;

use std::str;
use std::str::FromStr;
use std::str::Utf8Error;

use crate::lexer::token::*;

macro_rules! syntax {
    ($func_name: ident, $tag_string: literal, $output_token: expr) => {
        fn $func_name(s: &[u8]) -> IResult<&[u8], Token> {
            map(tag($tag_string), |_| $output_token)(s)
        }        
    };
}

// operators
syntax! {equal_operator, "==", Token::Equal}
syntax! {not_equal_operator, "!=", Token::NotEqual}
syntax! {assign_operator, "=", Token::Assign}
syntax! {plus_operator, "+", Token::Plus}
syntax! {minus_operator, "-", Token::Minus}
syntax! {multiply_operator, "*", Token::Multiply}
syntax! {divide_operator, "/", Token::Divide}
syntax! {not_operator, "!", Token::Not}
syntax! {greater_operator_equal, ">=", Token::GreaterThanEqual}
syntax! {lesser_operator_equal, "<=", Token::LessThanEqual}
syntax! {greater_operator, ">", Token::GreaterThan}
syntax! {lesser_operator, "<", Token::LessThan}
syntax! {and_operator, "&&", Token::And}
syntax! {or_operator,  "||", Token::Or}

pub fn lex_operator(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        equal_operator,
        not_equal_operator,
        assign_operator,
        plus_operator,
        minus_operator,
        multiply_operator,
        divide_operator,
        not_operator,
        greater_operator_equal,
        lesser_operator_equal,
        greater_operator,
        lesser_operator,
        and_operator,
        or_operator,
    ))(input)
}

syntax! {comma_punctuation, ",", Token::Comma}
syntax! {semicolon_punctuation, ";", Token::SemiColon}
syntax! {colon_punctuation, ":", Token::Colon}
syntax! {lparen_punctuation, "(", Token::LParen}
syntax! {rparen_punctuation, ")", Token::RParen}
syntax! {lbrace_punctuation, "{", Token::LBrace}
syntax! {rbrace_punctuation, "}", Token::RBrace}
syntax! {lbracket_punctuation, "[", Token::LBracket}
syntax! {rbracket_punctuation, "]", Token::RBracket}
syntax! {dot_punctuation, ".", Token::Dot}

pub fn lex_punctuations(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        comma_punctuation,
        semicolon_punctuation,
        colon_punctuation,
        lparen_punctuation,
        rparen_punctuation,
        lbrace_punctuation,
        rbrace_punctuation,
        lbracket_punctuation,
        rbracket_punctuation,
        dot_punctuation
    ))(input)
}

// String parsing
fn parse_escaped_char(input: &[u8]) -> IResult<&[u8], char> {
    preceded(
        tag("\\"),
        alt((
            value('"', tag("\"")),
            value('\\', tag("\\")),
            value('n', tag("n")),
            value('r', tag("r")),
            value('t', tag("t")),
        ))
    )(input)
}

fn parse_string_fragment(input: &[u8]) -> IResult<&[u8], String> {
    alt((
        map(parse_escaped_char, |c| {
            match c {
                'n' => "\n".to_string(),
                'r' => "\r".to_string(),
                't' => "\t".to_string(),
                other => other.to_string(),
            }
        }),
        map_res(
            is_not("\"\\"),
            |bytes: &[u8]| str::from_utf8(bytes).map(|s| s.to_string())
        ),
    ))(input)
}

fn parse_string_contents(input: &[u8]) -> IResult<&[u8], String> {
    let (input, fragments) = many0(parse_string_fragment)(input)?;
    Ok((input, fragments.join("")))
}

fn lex_string(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        delimited(tag("\""), parse_string_contents, tag("\"")),
        Token::StringLiteral
    )(input)
}

fn complete_byte_slice_str_from_utf8(c: &[u8]) -> Result<&str, Utf8Error> {
    str::from_utf8(c)
}

// Reserved or ident
fn lex_reserved_ident(input: &[u8]) -> IResult<&[u8], Token> {
    map_res(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s| {
            let c = complete_byte_slice_str_from_utf8(s);
            c.map(|syntax| match syntax {
                "let" => Token::Let,
                "fn" => Token::Function,
                "if" => Token::If,
                "else" => Token::Else,
                "return" => Token::Return,
                "struct" => Token::Struct,
                "this" => Token::This,
                "import" => Token::Import,
                "true" => Token::BoolLiteral(true),
                "false" => Token::BoolLiteral(false),
                "null" => Token::NullLiteral,
                "while" => Token::While,
                "for" => Token::For,
                "in" => Token::In,
                "break" => Token::Break,
                "continue" => Token::Continue,
                _ => Token::Ident(syntax.to_string()),
            })
        },
    )(input)
}

fn complete_str_from_str<F: FromStr>(c: &str) -> Result<F, F::Err> {
    FromStr::from_str(c)
}

// Integers parsing
fn lex_integer(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        map_res(
            map_res(digit1, complete_byte_slice_str_from_utf8),
            complete_str_from_str,
        ),
        Token::IntLiteral,
    )(input)
}

// Illegal tokens
fn lex_illegal(input: &[u8]) -> IResult<&[u8], Token> {
    map(take(1usize), |_| Token::Illegal)(input)
}

fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        lex_string,
        lex_operator,
        lex_punctuations,
        lex_reserved_ident,
        lex_integer,
        lex_illegal,
    ))(input)
}

fn skip_line_comment(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _) = tag("//")(input)?;
    let (input, _) = opt(is_not("\n\r"))(input)?;
    let (input, _) = opt(line_ending)(input)?;
    Ok((input, ()))
}

fn skip_ws_and_comments(input: &[u8]) -> IResult<&[u8], ()> {
    let (mut input, _) = multispace0(input)?;
    
    loop {
        if let Ok((remaining, _)) = skip_line_comment(input) {
            let (remaining, _) = multispace0(remaining)?;
            input = remaining;
        } else {
            break;
        }
    }
    
    Ok((input, ()))
}

fn lex_tokens(input: &[u8]) -> IResult<&[u8], Vec<Token>> {
    let (mut input, _) = skip_ws_and_comments(input)?;
    let mut tokens = Vec::new();
    
    loop {
        // Try to lex a token
        if input.is_empty() {
            break;
        }
        
        match lex_token(input) {
            Ok((remaining, token)) => {
                tokens.push(token);
                input = remaining;
                
                let (remaining, _) = skip_ws_and_comments(input)?;
                input = remaining;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok((input, tokens))
}

pub struct Lexer;

impl Lexer {
    pub fn lex_tokens(bytes: &[u8]) -> IResult<&[u8], Vec<Token>> {
        lex_tokens(bytes)
            .map(|(slice, result)| (slice, [&result[..], &vec![Token::EOF][..]].concat()))
    }
}