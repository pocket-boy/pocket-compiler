use nom::{
    IResult, Parser as NomParser,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{alpha1, digit1, line_ending},
    combinator::{opt, recognize},
    multi::{many_m_n, many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};
use nom_supreme::{ParserExt as _, error::ErrorTree, tag::complete::tag};

/// Result type for parsing.
pub type Result<'s, T> = IResult<&'s str, T, ErrorTree<&'s str>>;

/// Current parser state.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Parser(pub usize);

/// Represents an expression.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expr<'s>(pub &'s str);

/// Represents an identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name<'s>(pub &'s str);

/// Represents a function definition.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Func<'s>(
    pub Name<'s>,
    pub Name<'s>,
    pub Vec<(Name<'s>, Name<'s>)>,
    Body<'s>,
);

/// Represents a code block.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body<'s>(pub Vec<Stmt<'s>>);

/// Represents a top-level item.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item<'s>(pub bool, pub Stmt<'s>);

/// Reprents a component of a code block.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stmt<'s> {
    /// ...
    Bind(Bind<'s>),
    /// ...
    Func(Func<'s>),
}

/// Represents an assignment.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bind<'s> {
    /// ...
    Let(Name<'s>, Option<Name<'s>>, Expr<'s>),
    /// ...
    Var(Name<'s>, Option<Name<'s>>, Expr<'s>),
}

impl Parser {
    /// ...
    fn space(input: char) -> bool {
        input == ' ' || input == '\t'
    }
}

impl Parser {
    /// ...
    pub fn expr(input: &str) -> Result<'_, Expr<'_>> {
        // NOTE: Currently, a boolean or number.
        alt((tag("true"), tag("false"), digit1))
            .map(Expr)
            .parse(input)
    }

    /// ...
    pub fn name(input: &str) -> Result<'_, Name<'_>> {
        // Recognise identifier pattern.
        recognize(tuple((
            alt((tag("_"), alpha1)),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )))
        .map(Name)
        .parse(input)
    }

    /// ...
    pub fn item(input: &str) -> Result<'_, Item<'_>> {
        // Parse optional visibility specifier.
        let (input, local) =
            opt(terminated(tag("local"), take_while1(Self::space))).parse(input)?;
        // Parse item content.
        Self::default()
            .stmt()
            .map(|stmt| Item(local.is_some(), stmt))
            .parse(input)
    }

    /// ...
    pub fn bind(input: &str) -> Result<'_, Bind<'_>> {
        // Parse binding kind.
        let (input, kind) = alt((tag("let"), tag("var"))).parse(input)?;
        // Parse binding identifier.
        let (input, name) = preceded(take_while1(Self::space), Self::name).parse(input)?;
        // Parse optional type annotation.
        let (input, anno) = preceded(
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
        )
        .parse(input)?;
        // Parse binding assignment value.
        let (input, expr) = preceded(take_while(Self::space), Self::expr).parse(input)?;
        // Return as success.
        Ok((
            input,
            match kind {
                "let" => Bind::Let(name, anno, expr),
                "var" => Bind::Var(name, anno, expr),
                _ => unreachable!(),
            },
        ))
    }
}

impl Parser {
    /// ...
    pub fn stmt<'s>(&self) -> impl NomParser<&'s str, Stmt<'s>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::stmt()`.
        move |input| {
            // NOTE: Currently, either a function definition or binding.
            alt((self.func().map(Stmt::Func), Self::bind.map(Stmt::Bind))).parse(input)
        }
    }

    /// ...
    pub fn func<'s>(&self) -> impl NomParser<&'s str, Func<'s>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::func()`.
        move |input| {
            // Parse function identifier.
            let (input, name) =
                preceded(tuple((tag("def"), take_while1(Self::space))), Self::name).parse(input)?;
            // Parse function parameter list.
            let (input, params) = delimited(
                tuple((take_while(Self::space), tag("("))),
                separated_list0(
                    tag(","),
                    delimited(
                        take_while(Self::space),
                        separated_pair(
                            Self::name,
                            tuple((take_while(Self::space), tag(":"), take_while(Self::space))),
                            Self::name,
                        ),
                        take_while(Self::space),
                    ),
                ),
                tag(")"),
            )
            .parse(input)?;
            // Parse function type annotation.
            let (input, anno) = delimited(
                tuple((take_while(Self::space), tag("->"), take_while(Self::space))),
                Self::name,
                tuple((take_while(Self::space), tag(":"), line_ending)),
            )
            .parse(input)?;
            // Parse function body block.
            let (input, body) = self.body().parse(input)?;
            // Return as success.
            Ok((input, Func(name, anno, params, body)))
        }
    }

    /// ...
    pub fn body<'s>(&self) -> impl NomParser<&'s str, Body<'s>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::body()`.
        move |input| {
            // ...
            let (input, count) = preceded(
                many0(tuple((take_while(Self::space), line_ending))),
                many_m_n(self.0 + 1, usize::MAX, alt((tag(" "), tag("\t"))))
                    .map(|result| result.len()),
            )
            .parse(input)?;
            // ...
            let (input, first) = Self(count).stmt().parse(input)?;
            // ...
            let (input, items) = many0(preceded(
                preceded(
                    many0(tuple((take_while(Self::space), line_ending))),
                    many_m_n(count, count, alt((tag(" "), tag("\t")))),
                ),
                Self(count).stmt(),
            ))
            .parse(input)?;
            // ...
            Ok((
                input,
                Body(std::iter::once(first.clone()).chain(items).collect()),
            ))
        }
    }
}
