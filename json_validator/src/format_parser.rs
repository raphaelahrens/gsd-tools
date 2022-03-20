use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, none_of},
    combinator::{eof, map, opt, recognize, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::{many0_count, many1},
    number::complete::double,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::error::Error;
use std::str;

#[derive(Debug, PartialEq, Eq)]
struct Indention {
    count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    OpenCurley,
    CloseCurley,
    OpenSquare,
    CloseSquare,
    Value,
}
fn indention(i: &str) -> IResult<&str, Indention> {
    map(many0_count(char(' ')), |count| Indention { count })(i)
}

fn open_curley(i: &str) -> IResult<&str, Token> {
    value(Token::OpenCurley, char('{'))(i)
}
fn close_curley(i: &str) -> IResult<&str, Token> {
    value(Token::CloseCurley, char('}'))(i)
}
fn open_square(i: &str) -> IResult<&str, Token> {
    value(Token::OpenSquare, char('['))(i)
}
fn close_square(i: &str) -> IResult<&str, Token> {
    value(Token::CloseSquare, char(']'))(i)
}

fn close_eol(i: &str) -> IResult<&str, ()> {
    map(tuple((opt(char(',')), line_ending)), |_t| ())(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(many0_count(none_of("\"")))(i)
}

fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('\"'), terminated(parse_str, char('\"')))(i)
}

fn member_key(i: &str) -> IResult<&str, ()> {
    map(terminated(string, tag(": ")), |_s| ())(i)
}

fn json_value(i: &str) -> IResult<&str, Token> {
    value(
        Token::Value,
        alt((
            recognize(string),
            recognize(double),
            recognize(tag("null")),
            recognize(tag("true")),
            recognize(tag("false")),
            recognize(tag("{}")),
            recognize(tag("[]")),
        )),
    )(i)
}

fn open_line(i: &str) -> IResult<&str, (Indention, Token)> {
    terminated(
        separated_pair(indention, opt(member_key), alt((open_curley, open_square))),
        line_ending,
    )(i)
}
fn close_line(i: &str) -> IResult<&str, (Indention, Token)> {
    terminated(
        tuple((indention, alt((close_curley, close_square)))),
        alt((close_eol, value((),eof))),
    )(i)
}

fn member_line(i: &str) -> IResult<&str, (Indention, Token)> {
    terminated(separated_pair(indention, member_key, json_value), close_eol)(i)
}

fn line(i: &str) -> IResult<&str, (Indention, Token)> {
    alt((open_line, close_line, member_line))(i)
}

fn parse(i: &str) -> IResult<&str, Vec<(Indention, Token)>> {
    many1(line)(i)
}

pub fn check_format(i: &str) -> bool {
    let (_rest, lines) = match parse(&i) {
        Err(e) => {
            return false;
        }
        Ok(lines) => lines,
    };
    let mut indent_level = 0;
    for (indent, token) in lines {
        //dbg!(&indent.count);
        //dbg!(&indent_level);
        //dbg!(&token);
        match token {
            Token::OpenSquare | Token::OpenCurley => {
                if indent.count != 4 * indent_level {
                    return false;
                }
                indent_level += 1;
            }
            Token::CloseCurley | Token::CloseSquare => {
                indent_level -= 1;
                if indent.count != 4 * indent_level {
                    return false;
                }
            }
            Token::Value => {
                if indent.count != 4 * indent_level {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oc_member_line_test() {
        let json_str = "       \"foo\": {\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::OpenCurley)))
        );
    }
    #[test]
    fn oc_line_test() {
        let json_str = "       {\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::OpenCurley)))
        );
    }
    #[test]
    fn cc_member_line_test() {
        let json_str = "       }\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::CloseCurley)))
        );
    }
    #[test]
    fn cc_line_test() {
        let json_str = "       },\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::CloseCurley)))
        );
    }
    #[test]
    fn os_member_line_test() {
        let json_str = "       \"foo\": [\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::OpenSquare)))
        );
    }
    #[test]
    fn os_line_test() {
        let json_str = "       [\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::OpenSquare)))
        );
    }
    #[test]
    fn member_null() {
        let json_str = "       \"member\": null\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::Value)))
        );
    }
    #[test]
    fn member_null_comma() {
        let json_str = "       \"member\": null,\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::Value)))
        );
    }
    #[test]
    fn member_number_comma() {
        let json_str = "       \"member\": 10,\n";
        assert_eq!(
            line(json_str),
            Ok(("", (Indention { count: 7 }, Token::Value)))
        );
        let json_str2 = "       \"member\": 10.0,\n";
        assert_eq!(
            line(json_str2),
            Ok(("", (Indention { count: 7 }, Token::Value)))
        );
        let json_str3 = "       \"member\": 12e-2,\n";
        assert_eq!(
            line(json_str3),
            Ok(("", (Indention { count: 7 }, Token::Value)))
        );
    }
}
