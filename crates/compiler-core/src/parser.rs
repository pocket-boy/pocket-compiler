use std::cmp::Ordering;

use derive_more::{From, TryInto};
use nom::{
    IResult, Parser as NomParser,
    branch::alt,
    bytes::complete::{is_a, is_not, take_while, take_while1},
    character::complete::{alpha1, digit1, line_ending},
    combinator::{all_consuming, opt, recognize},
    multi::{many_m_n, many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};
use nom_supreme::{ParserExt as _, error::ErrorTree, tag::complete::tag};

/// Result type for parsing.
pub type Result<'s, T> = IResult<&'s str, T, ErrorTree<&'s str>>;

////////////////////////////////////////////////////////////

/// ...
pub type Borrowed<'s> = &'s str;

/// ...
pub type Owned = String;

////////////////////////////////////////////////////////////

/// ...
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Parser(pub usize);

////////////////////////////////////////////////////////////

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Num(pub isize);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Str(pub String);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Arr(pub usize);

////////////////////////////////////////////////////////////

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name<S>(pub S);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<S>(pub Vec<Name<S>>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Call<S>(pub Path<S>, pub Vec<Expr<S>>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct File<S>(pub Vec<Item<S>>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Assn<S>(pub Name<S>, pub Expr<S>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bind<S> {
    /// ...
    Let(Name<S>, Option<Path<S>>, Expr<S>),
    /// ...
    Var(Name<S>, Option<Path<S>>, Expr<S>),
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item<S> {
    /// ...
    Bind(bool, Bind<S>),
    /// ...
    Func(bool, Func<S>),
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, TryInto, From)]
pub enum Prim<S> {
    /// ...
    Arr(Arr),
    /// ...
    Num(Num),
    /// ...
    Str(Str),
    /// ...
    Bool(bool),
    /// ...
    Path(Path<S>),
    /// ...
    Call(Call<S>),
    /// ...
    Sect(Box<Expr<S>>),
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr<S> {
    /// ...
    Prim(Prim<S>),
    /// ...
    Comp(Prim<S>),
    /// ...
    And(Prim<S>, Prim<S>),
    /// ...
    Orr(Prim<S>, Prim<S>),
    /// ...
    Add(Prim<S>, Prim<S>),
    /// ...
    Sub(Prim<S>, Prim<S>),
    /// ...
    Mul(Prim<S>, Prim<S>),
    /// ...
    BitShl(Prim<S>, Prim<S>),
    /// ...
    BitAnd(Prim<S>, Prim<S>),
    /// ...
    BitOrr(Prim<S>, Prim<S>),
    /// ...
    Ord(bool, Ordering, Prim<S>, Prim<S>),
}

////////////////////////////////////////////////////////////

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cond<S>(pub Expr<S>, pub Body<S>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loop<S>(pub Expr<S>, pub Body<S>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body<S>(pub Vec<Stmt<S>>);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Func<S>(
    pub Name<S>,
    pub Path<S>,
    pub Vec<(Name<S>, Path<S>)>,
    pub Body<S>,
);

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From, TryInto)]
pub enum Stmt<S> {
    /// ...
    Bind(Bind<S>),
    /// ...
    Assn(Assn<S>),
    /// ...
    Func(Func<S>),
    /// ...
    Call(Call<S>),
    /// ...
    Cond(Cond<S>),
    /// ...
    Loop(Loop<S>),
    /// ...
    #[from(ignore)]
    Ret(Option<Expr<S>>),
}

////////////////////////////////////////////////////////////

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
        // digit1
        //     .map(|n: &str| Num(n.parse().expect("number literal too large")))
        //     .parse(input)
        alt((
            // ...
            preceded(tag("0b"), is_a("01"))
                .map(|n| Num(isize::from_str_radix(n, 2).expect("failed to parse integer"))),
            // ...
            preceded(tag("0o"), is_a("01234567"))
                .map(|n| Num(isize::from_str_radix(n, 8).expect("failed to parse integer"))),
            // ...
            preceded(tag("0x"), is_a("01234567abcdefABCDEF"))
                .map(|n| Num(isize::from_str_radix(n, 16).expect("failed to parse integer"))),
            // ...
            digit1.map(|n| Num(isize::from_str_radix(n, 10).expect("failed to parse integer"))),
        ))
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
    pub fn arr(input: &str) -> Result<'_, Arr> {
        // ...
        delimited(
            tuple((tag("["), take_while(Self::space))),
            Self::num,
            tuple((take_while(Self::space), tag("]"))),
        )
        .map(|Num(num)| Arr(num as usize))
        .parse(input)
    }
}

impl Parser {
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
    pub fn path(input: &str) -> Result<'_, Path<Borrowed<'_>>> {
        // ...
        separated_list1(tag("."), Self::name).map(Path).parse(input)
    }

    /// ...
    pub fn call(input: &str) -> Result<'_, Call<Borrowed<'_>>> {
        // ...
        let (input, path) = terminated(Self::path, take_while(Self::space)).parse(input)?;
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
        Ok((input, Call(path, args)))
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

    /// ...
    pub fn assn(input: &str) -> Result<'_, Assn<Borrowed<'_>>> {
        // ...
        let (input, name) = Self::name.parse(input)?;
        // ...
        let (input, _) =
            tuple((take_while(Self::space), tag("="), take_while(Self::space))).parse(input)?;
        // ...
        let (input, expr) = Self::expr.parse(input)?;
        // ...
        Ok((input, Assn(name, expr)))
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
                    Self::path,
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
    pub fn item(input: &str) -> Result<'_, Item<Borrowed<'_>>> {
        // Parse optional visibility specifier.
        let (input, local) =
            opt(terminated(tag("local"), take_while1(Self::space))).parse(input)?;
        // Parse item content.
        alt((
            Self::bind.map(|bind| Item::Bind(local.is_some(), bind)),
            Self::default()
                .func()
                .map(|func| Item::Func(local.is_some(), func)),
        ))
        .parse(input)
    }

    /// ...
    pub fn prim(input: &str) -> Result<'_, Prim<Borrowed<'_>>> {
        // NOTE: Currently, a boolean, number, identifier, or call expression.
        alt((
            // ...
            delimited(
                tuple((tag("("), take_while(Self::space))),
                Self::expr,
                tuple((take_while(Self::space), tag(")"))),
            )
            .map(|expr| Prim::Sect(Box::new(expr))),
            // ...
            tag("true").value(Prim::Bool(true)),
            tag("false").value(Prim::Bool(false)),
            Self::arr.map(From::from),
            Self::str.map(From::from),
            Self::num.map(From::from),
            Self::call.map(From::from),
            Self::path.map(From::from),
        ))
        .parse(input)
    }

    /// ...
    pub fn expr(input: &str) -> Result<'_, Expr<Borrowed<'_>>> {
        // TODO: implement `Parser::expr()`.
        alt((
            // ...
            preceded(tuple((tag("!"), take_while(Self::space))), Self::prim).map(Expr::Comp),
            // ...
            tuple((
                Self::prim,
                delimited(
                    take_while(Self::space),
                    alt((
                        tag("+"),
                        tag("-"),
                        tag("*"),
                        tag("=="),
                        tag("!="),
                        tag(">="),
                        tag("<="),
                        tag(">"),
                        tag("<"),
                        tag("&&"),
                        tag("||"),
                        tag("&"),
                        tag("|"),
                    )),
                    take_while(Self::space),
                ),
                Self::prim,
            ))
            .map(|(lhs, op, rhs)| match op {
                "+" => Expr::Add(lhs, rhs),
                "-" => Expr::Sub(lhs, rhs),
                "*" => Expr::Mul(lhs, rhs),
                "&&" => Expr::And(lhs, rhs),
                "||" => Expr::Orr(lhs, rhs),
                "&" => Expr::BitAnd(lhs, rhs),
                "|" => Expr::BitOrr(lhs, rhs),
                ">" => Expr::Ord(false, Ordering::Greater, lhs, rhs),
                "<" => Expr::Ord(false, Ordering::Less, lhs, rhs),
                "==" => Expr::Ord(false, Ordering::Equal, lhs, rhs),
                ">=" => Expr::Ord(true, Ordering::Less, lhs, rhs),
                "<=" => Expr::Ord(true, Ordering::Greater, lhs, rhs),
                "!=" => Expr::Ord(true, Ordering::Equal, lhs, rhs),
                _ => unreachable!(),
            }),
            // ...
            Self::prim.map(Expr::Prim),
        ))
        .parse(input)
    }
}

impl Parser {
    /// ...
    pub fn cond<'s>(&self) -> impl NomParser<&'s str, Cond<Borrowed<'s>>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::cond()`.
        move |input| {
            // ...
            let (input, cond) = delimited(
                tuple((tag("if"), take_while1(Self::space))),
                Self::expr,
                tuple((take_while(Self::space), tag(":"), line_ending)),
            )
            .parse(input)?;
            // ...
            let (input, body) = self.body().parse(input)?;
            // ...
            Ok((input, Cond(cond, body)))
        }
    }

    /// ...
    pub fn fold<'s>(&self) -> impl NomParser<&'s str, Loop<Borrowed<'s>>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::fold()`.
        move |input| {
            // ...
            let (input, cond) = delimited(
                tuple((tag("while"), take_while1(Self::space))),
                Self::expr,
                tuple((take_while(Self::space), tag(":"), line_ending)),
            )
            .parse(input)?;
            // ...
            let (input, body) = self.body().parse(input)?;
            // ...
            Ok((input, Loop(cond, body)))
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
                            Self::path,
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
                Self::path,
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
    pub fn stmt<'s>(&self) -> impl NomParser<&'s str, Stmt<Borrowed<'s>>, ErrorTree<&'s str>> {
        // TODO: implement `Parser::stmt()`.
        move |input| {
            // NOTE: Currently, either a function definition, return statement, call expression, or binding.
            alt((
                // preceded(tuple((tag("return"), take_while(Self::space))), Self::expr)
                //     .map(Stmt::Ret),
                // ...
                tuple((
                    tag("return"),
                    opt(preceded(take_while(Self::space), Self::expr)),
                ))
                .map(|(_, expr)| Stmt::Ret(expr)),
                // ...
                self.func().map(From::from),
                self.cond().map(From::from),
                self.fold().map(From::from),
                Self::bind.map(From::from),
                Self::assn.map(From::from),
                Self::call.map(From::from),
            ))
            .parse(input)
        }
    }
}

impl From<Name<Borrowed<'_>>> for Name<Owned> {
    fn from(value: Name<Borrowed<'_>>) -> Self {
        Self(value.0.into())
    }
}

impl From<Path<Borrowed<'_>>> for Path<Owned> {
    fn from(value: Path<Borrowed<'_>>) -> Self {
        Self(value.0.into_iter().map(From::from).collect())
    }
}

impl From<Call<Borrowed<'_>>> for Call<Owned> {
    fn from(value: Call<Borrowed<'_>>) -> Self {
        Self(
            value.0.into(),
            value.1.into_iter().map(From::from).collect(),
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

impl From<Cond<Borrowed<'_>>> for Cond<Owned> {
    fn from(value: Cond<Borrowed<'_>>) -> Self {
        Self(value.0.into(), value.1.into())
    }
}

impl From<Loop<Borrowed<'_>>> for Loop<Owned> {
    fn from(value: Loop<Borrowed<'_>>) -> Self {
        Self(value.0.into(), value.1.into())
    }
}

impl From<Body<Borrowed<'_>>> for Body<Owned> {
    fn from(value: Body<Borrowed<'_>>) -> Self {
        Self(value.0.into_iter().map(|stmt| stmt.into()).collect())
    }
}

impl From<Item<Borrowed<'_>>> for Item<Owned> {
    fn from(value: Item<Borrowed<'_>>) -> Self {
        match value {
            Item::Bind(local, bind) => Self::Bind(local, bind.into()),
            Item::Func(local, func) => Self::Func(local, func.into()),
        }
    }
}

impl From<File<Borrowed<'_>>> for File<Owned> {
    fn from(value: File<Borrowed<'_>>) -> Self {
        Self(value.0.into_iter().map(|item| item.into()).collect())
    }
}

impl From<Assn<Borrowed<'_>>> for Assn<Owned> {
    fn from(value: Assn<Borrowed<'_>>) -> Self {
        Self(value.0.into(), value.1.into())
    }
}

impl From<Prim<Borrowed<'_>>> for Prim<Owned> {
    fn from(value: Prim<Borrowed<'_>>) -> Self {
        match value {
            Prim::Arr(arr) => Self::Arr(arr),
            Prim::Num(num) => Self::Num(num),
            Prim::Str(str) => Self::Str(str),
            Prim::Bool(bool) => Self::Bool(bool),
            Prim::Path(path) => Self::Path(path.into()),
            Prim::Call(call) => Self::Call(call.into()),
            Prim::Sect(sect) => Self::Sect(Box::new(Expr::<Owned>::from(*sect))),
        }
    }
}

impl From<Expr<Borrowed<'_>>> for Expr<Owned> {
    fn from(value: Expr<Borrowed<'_>>) -> Self {
        match value {
            Expr::Prim(prim) => Self::Prim(prim.into()),
            Expr::Comp(comp) => Self::Comp(comp.into()),
            Expr::And(lhs, rhs) => Self::And(lhs.into(), rhs.into()),
            Expr::Orr(lhs, rhs) => Self::Orr(lhs.into(), rhs.into()),
            Expr::Add(lhs, rhs) => Self::Add(lhs.into(), rhs.into()),
            Expr::Sub(lhs, rhs) => Self::Sub(lhs.into(), rhs.into()),
            Expr::Mul(lhs, rhs) => Self::Mul(lhs.into(), rhs.into()),
            Expr::BitShl(lhs, rhs) => Self::BitShl(lhs.into(), rhs.into()),
            Expr::BitAnd(lhs, rhs) => Self::BitAnd(lhs.into(), rhs.into()),
            Expr::BitOrr(lhs, rhs) => Self::BitOrr(lhs.into(), rhs.into()),
            Expr::Ord(comp, ord, lhs, rhs) => Self::Ord(comp, ord, lhs.into(), rhs.into()),
        }
    }
}

impl From<Stmt<Borrowed<'_>>> for Stmt<Owned> {
    fn from(value: Stmt<Borrowed<'_>>) -> Self {
        match value {
            Stmt::Bind(bind) => Self::Bind(bind.into()),
            Stmt::Assn(assn) => Self::Assn(assn.into()),
            Stmt::Func(func) => Self::Func(func.into()),
            Stmt::Call(call) => Self::Call(call.into()),
            Stmt::Cond(cond) => Self::Cond(cond.into()),
            Stmt::Loop(fold) => Self::Loop(fold.into()),
            Stmt::Ret(expr) => Self::Ret(expr.map(From::from)),
        }
    }
}

impl From<Bind<Borrowed<'_>>> for Bind<Owned> {
    fn from(value: Bind<Borrowed<'_>>) -> Self {
        match value {
            Bind::Let(name, anno, expr) => {
                Self::Let(name.into(), anno.map(From::from), expr.into())
            }
            Bind::Var(name, anno, expr) => {
                Self::Var(name.into(), anno.map(From::from), expr.into())
            }
        }
    }
}
