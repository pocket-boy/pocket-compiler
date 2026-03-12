use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{alpha1, digit1},
    combinator::{opt, recognize},
    sequence::{delimited, preceded, terminated, tuple},
};
use nom_supreme::{ParserExt, error::ErrorTree, tag::complete::tag};

/// ...
pub type Result<'s, T> = IResult<&'s str, T, ErrorTree<&'s str>>;

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Parser(pub usize);

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item<'s>(pub bool, pub Bind<'s>);

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expr<'s>(pub &'s str);

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name<'s>(pub &'s str);

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bind<'s> {
    /// ...
    Let(Name<'s>, Option<Name<'s>>, Expr<'s>),
    /// ...
    Var(Name<'s>, Option<Name<'s>>, Expr<'s>),
}

impl Parser {
    /// ...
    pub fn space(input: char) -> bool {
        input == ' ' || input == '\t'
    }
}

impl Parser {
    /// ...
    pub fn item(input: &str) -> Result<'_, Item<'_>> {
        tuple((
            opt(terminated(tag("local"), take_while1(Self::space))),
            Self::bind,
        ))
        .map(|(local, bind)| Item(local.is_some(), bind))
        .parse(input)
    }

    /// ...
    pub fn expr(input: &str) -> Result<'_, Expr<'_>> {
        alt((tag("true"), tag("false"), digit1))
            .map(Expr)
            .parse(input)
    }

    /// ...
    pub fn name(input: &str) -> Result<'_, Name<'_>> {
        recognize(tuple((
            alt((tag("_"), alpha1)),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )))
        .map(Name)
        .parse(input)
    }

    /// ...
    pub fn bind(input: &str) -> Result<'_, Bind<'_>> {
        tuple((
            alt((tag("let"), tag("var"))),
            tuple((
                preceded(take_while1(Self::space), Self::name),
                preceded(
                    take_while(Self::space),
                    alt((
                        delimited(
                            tuple((tag(":"), take_while(Self::space))),
                            Self::name,
                            tuple((take_while(Self::space), tag("="))),
                        )
                        .map(Some),
                        tag(":=").value(None),
                    )),
                ),
                preceded(take_while(Self::space), Self::expr),
            )),
        ))
        .map(|(kind, (name, anno, expr))| match kind {
            "let" => Bind::Let(name, anno, expr),
            "var" => Bind::Var(name, anno, expr),
            _ => unreachable!(),
        })
        .parse(input)
    }
}
