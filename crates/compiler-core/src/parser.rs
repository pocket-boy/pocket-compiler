use nom::{
    IResult, Parser as NomParser,
    branch::alt,
    bytes::complete::{is_not, take_while, take_while1},
    character::complete::{alpha1, digit1, line_ending},
    combinator::{all_consuming, opt, recognize},
    multi::{many_m_n, many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};
use nom_supreme::{ParserExt as _, error::ErrorTree, tag::complete::tag};

/// ...
pub type Borrowed<'s> = &'s str;

/// ...
pub type Owned = String;

/// Result type for parsing.
pub type Result<'s, T> = IResult<&'s str, T, ErrorTree<&'s str>>;

/// Number literal.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Num(pub usize);

/// String literal.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Str(pub String);

/// Current parser state.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Parser(pub usize);

/// Represents an identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name<S>(pub S);

/// Represents an identifier.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call<S>(pub Name<S>, pub Vec<Expr<S>>);

/// Represents a function definition.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Func<S>(
    pub Name<S>,
    pub Name<S>,
    pub Vec<(Name<S>, Name<S>)>,
    pub Body<S>,
);

/// Represents a code block.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body<S>(pub Vec<Stmt<S>>);

/// Represents a top-level item.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item<S>(pub bool, pub Stmt<S>);

/// Represents a file-based module.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct File<S>(pub Vec<Item<S>>);

/// Represents an expression.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr<S> {
    /// ...
    Num(Num),
    /// ...
    Str(Str),
    /// ...
    Bool(bool),
    /// ...
    Name(Name<S>),
    /// ...
    Call(Call<S>),
}

/// Reprents a component of a code block.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stmt<S> {
    /// ...
    Bind(Bind<S>),
    /// ...
    Func(Func<S>),
    /// ...
    Call(Call<S>),
    /// ...
    Ret(Expr<S>),
}

/// Represents an assignment.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bind<S> {
    /// ...
    Let(Name<S>, Option<Name<S>>, Expr<S>),
    /// ...
    Var(Name<S>, Option<Name<S>>, Expr<S>),
}

impl Parser {
    /// ...
    fn space(input: char) -> bool {
        input == ' ' || input == '\t'
    }
}

impl Parser {
    /// ...
    pub fn num(input: &str) -> Result<'_, Num> {
        // WARN: Currently, throws on too large. Fix for later!
        digit1
            .map(|n: &str| Num(n.parse().expect("number literal too large")))
            .parse(input)
    }

    /// ...
    pub fn str(input: &str) -> Result<'_, Str> {
        // ...
        delimited(tag("\""), is_not("\""), tag("\""))
            .map(|s| Str(String::from(s)))
            .parse(input)
    }

    /// ...
    pub fn expr(input: &str) -> Result<'_, Expr<Borrowed<'_>>> {
        // NOTE: Currently, a boolean, number, identifier, or call expression.
        alt((
            tag("true").value(Expr::Bool(true)),
            tag("false").value(Expr::Bool(false)),
            Self::str.map(Expr::Str),
            Self::num.map(Expr::Num),
            Self::call.map(Expr::Call),
            Self::name.map(Expr::Name),
        ))
        .parse(input)
    }

    /// ...
    pub fn name(input: &str) -> Result<'_, Name<Borrowed<'_>>> {
        // Recognise identifier pattern.
        recognize(tuple((
            alt((tag("_"), alpha1)),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )))
        .map(Name)
        .parse(input)
    }

    /// ...
    pub fn call(input: &str) -> Result<'_, Call<Borrowed<'_>>> {
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
    pub fn item(input: &str) -> Result<'_, Item<Borrowed<'_>>> {
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
    pub fn bind(input: &str) -> Result<'_, Bind<Borrowed<'_>>> {
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
    pub fn file(input: &str) -> Result<'_, File<Borrowed<'_>>> {
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
    pub fn stmt<'s>(&self) -> impl NomParser<&'s str, Stmt<Borrowed<'s>>, ErrorTree<&'s str>> {
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
    pub fn func<'s>(&self) -> impl NomParser<&'s str, Func<Borrowed<'s>>, ErrorTree<&'s str>> {
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
    pub fn body<'s>(&self) -> impl NomParser<&'s str, Body<Borrowed<'s>>, ErrorTree<&'s str>> {
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

impl From<Name<Borrowed<'_>>> for Name<Owned> {
    fn from(value: Name<Borrowed<'_>>) -> Self {
        Self(value.0.into())
    }
}

impl From<Call<Borrowed<'_>>> for Call<Owned> {
    fn from(value: Call<Borrowed<'_>>) -> Self {
        Self(
            value.0.into(),
            value.1.into_iter().map(|expr| expr.into()).collect(),
        )
    }
}

impl From<Func<Borrowed<'_>>> for Func<Owned> {
    fn from(value: Func<Borrowed<'_>>) -> Self {
        Self(
            value.0.into(),
            value.1.into(),
            value
                .2
                .into_iter()
                .map(|(name, anno)| (name.into(), anno.into()))
                .collect(),
            value.3.into(),
        )
    }
}

impl From<Body<Borrowed<'_>>> for Body<Owned> {
    fn from(value: Body<Borrowed<'_>>) -> Self {
        Self(value.0.into_iter().map(|stmt| stmt.into()).collect())
    }
}

impl From<Item<Borrowed<'_>>> for Item<Owned> {
    fn from(value: Item<Borrowed<'_>>) -> Self {
        Self(value.0, value.1.into())
    }
}

impl From<File<Borrowed<'_>>> for File<Owned> {
    fn from(value: File<Borrowed<'_>>) -> Self {
        Self(value.0.into_iter().map(|item| item.into()).collect())
    }
}

impl From<Expr<Borrowed<'_>>> for Expr<Owned> {
    fn from(value: Expr<Borrowed<'_>>) -> Self {
        match value {
            Expr::Num(num) => Self::Num(num),
            Expr::Str(str) => Self::Str(str),
            Expr::Bool(bool) => Self::Bool(bool),
            Expr::Name(name) => Self::Name(name.into()),
            Expr::Call(call) => Self::Call(call.into()),
        }
    }
}

impl From<Stmt<Borrowed<'_>>> for Stmt<Owned> {
    fn from(value: Stmt<Borrowed<'_>>) -> Self {
        match value {
            Stmt::Bind(bind) => Self::Bind(bind.into()),
            Stmt::Func(func) => Self::Func(func.into()),
            Stmt::Call(call) => Self::Call(call.into()),
            Stmt::Ret(expr) => Self::Ret(expr.into()),
        }
    }
}

impl From<Bind<Borrowed<'_>>> for Bind<Owned> {
    fn from(value: Bind<Borrowed<'_>>) -> Self {
        match value {
            Bind::Let(name, anno, expr) => {
                Self::Let(name.into(), anno.map(Name::into), expr.into())
            }
            Bind::Var(name, anno, expr) => {
                Self::Var(name.into(), anno.map(Name::into), expr.into())
            }
        }
    }
}
