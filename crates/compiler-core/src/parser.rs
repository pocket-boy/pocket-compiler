use nom::{
    IResult, Parser as NomParser,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{alpha1, digit1, line_ending},
    combinator::{all_consuming, eof, opt, recognize},
    multi::{many_m_n, many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};
use nom_supreme::{ParserExt as _, error::ErrorTree, tag::complete::tag};

/// Result type for parsing.
pub type Result<'s, T> = IResult<&'s str, T, ErrorTree<&'s str>>;

/// Current parser state.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Parser(pub usize);

/// Represents an identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name<'s>(pub &'s str);

/// Represents an identifier.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call<'s>(pub Name<'s>, pub Vec<Expr<'s>>);

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

/// Represents a file-based module.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct File<'s>(pub Vec<Item<'s>>);

/// Represents an expression.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr<'s> {
    /// ...
    Num(isize),
    /// ...
    Str(&'s str),
    /// ...
    Bool(bool),
    /// ...
    Name(Name<'s>),
    /// ...
    Call(Call<'s>),
}

/// Reprents a component of a code block.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stmt<'s> {
    /// ...
    Bind(Bind<'s>),
    /// ...
    Func(Func<'s>),
    /// ...
    Call(Call<'s>),
    /// ...
    Ret(Expr<'s>),
}

/// Represents an assignment.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
        // NOTE: Currently, a boolean, number, identifier, or call expression.
        alt((
            tag("true").value(Expr::Bool(true)),
            tag("false").value(Expr::Bool(false)),
            // WARN: Currently, throws on too large. Fix for later!
            digit1.map(|n: &str| Expr::Num(n.parse().expect("number literal too large"))),
            Self::call.map(Expr::Call),
            Self::name.map(Expr::Name),
        ))
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
    pub fn call(input: &str) -> Result<'_, Call<'_>> {
        // ...
        let (input, name) = terminated(Self::name, take_while(Self::space)).parse(input)?;
        // ...
        let (input, args) = delimited(
            tag("("),
            separated_list0(
                tag(","),
                delimited(take_while(Self::space), Self::expr, take_while(Self::space)),
            ),
            tag(")"),
        )
        .parse(input)?;
        // ...
        Ok((input, Call(name, args)))
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

    /// ...
    pub fn file(input: &str) -> Result<'_, File<'_>> {
        // ...
        // terminated(
        //     many0(alt((
        //         Self::item.map(Some),
        //         tuple((take_while(Self::space), line_ending)).value(None),
        //     ))),
        //     tuple((take_while(Self::space), eof)),
        // )
        // .map(|items| File(items.into_iter().flatten().collect()))
        // .parse(input)
        all_consuming(separated_list0(
            line_ending,
            alt((Self::item.map(Some), take_while(Self::space).value(None))),
        ))
        .map(|items| File(items.into_iter().flatten().collect()))
        .parse(input)
    }
}

impl Parser {
    /// ...
    pub fn stmt<'s>(&self) -> impl NomParser<&'s str, Stmt<'s>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::stmt()`.
        move |input| {
            // NOTE: Currently, either a function definition, return statement, or binding.
            alt((
                preceded(tuple((tag("return"), take_while(Self::space))), Self::expr)
                    .map(Stmt::Ret),
                self.func().map(Stmt::Func),
                Self::bind.map(Stmt::Bind),
                Self::call.map(Stmt::Call),
            ))
            .parse(input)
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
