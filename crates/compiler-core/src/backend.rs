use std::{cmp::Ordering, collections::HashMap, ops::ControlFlow};

use derive_more::{From, TryInto};
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::{compiler::Compiler, input::InputState, parser::*};

/// ...
pub trait Environ {
    /// ...
    fn action_one(&mut self, value: u32);
    /// ...
    fn action_two(&mut self, value: u32);
    /// ...
    fn draw_tile(&mut self, tile: u32, x: u32, y: u32);
    /// ...
    fn load_tile(&mut self, tiles: &String);
}

/// ...
pub type Scope = HashMap<Path<Owned>, Binding>;

/// ...
pub struct Backend<E> {
    /// ...
    pub compiler: Compiler,
    /// ...
    pub environ: E,
    /// ...
    pub scopes: Option<Vec<Scope>>,
    /// ...
    pub input: InputState,
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic {
    /// ...
    Draw,
    /// ...
    Pressed,
    /// ...
    ArrayGet,
    /// ...
    ArraySet,
    /// ...
    CharCode,
    /// ...
    LoadTilemap,
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From, TryInto)]
pub enum Value {
    /// ...
    Arr(Vec<Value>),
    /// ...
    Num(Num),
    /// ...
    Str(Str),
    /// ...
    Bool(bool),
    /// ...
    #[from(ignore)]
    #[try_into(ignore)]
    Unit,
    /// ...
    #[from(ignore)]
    #[try_into(ignore)]
    Window,
    /// ...
    #[from(ignore)]
    #[try_into(ignore)]
    Input,
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, From, TryInto)]
pub enum Binding {
    /// ...
    #[from(ignore)]
    #[try_into(ignore)]
    Value(bool, Value),
    /// ...
    Func(Func<Owned>),
    /// ...
    Intrinsic(Intrinsic),
}

impl<E: Environ> Backend<E> {
    /// ...
    pub fn new(compiler: Compiler, environ: E) -> Self {
        // ...
        let mut parser = final_parser::<_, _, _, ErrorTree<&str>>(Parser::file);
        // ...
        let Some(result) = compiler
            .modules
            .get("main")
            .and_then(|module| (parser(module)).map(File::<Owned>::from).ok())
        else {
            // ...
            return Self {
                compiler: compiler.clone(),
                environ,
                scopes: None,
                input: InputState(0),
            };
        };
        // ...
        let mut instance = Self {
            compiler: compiler.clone(),
            environ,
            scopes: None,
            input: InputState(0),
        };
        // ...
        instance.prepare(&result);
        // ...
        instance
    }

    /// ...
    fn prepare(&mut self, file: &File<Owned>) {
        // ...
        self.scopes = file
            .0
            .iter()
            .try_fold(vec![HashMap::new()], |mut scopes, item| {
                Self::eval_item(&mut self.environ, &mut scopes, item, InputState(0))?;
                Some(scopes)
            });
        // ...
        if let Some(scopes) = &mut self.scopes {
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![
                    Name(String::from("Window")),
                    Name(String::from("draw")),
                ]),
                Binding::Intrinsic(Intrinsic::Draw),
            );
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![
                    Name(String::from("Input")),
                    Name(String::from("pressed")),
                ]),
                Binding::Intrinsic(Intrinsic::Pressed),
            );
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![
                    Name(String::from("Window")),
                    Name(String::from("load_tilemap")),
                ]),
                Binding::Intrinsic(Intrinsic::LoadTilemap),
            );
            // ...
            for (offset, field) in ["RIGHT", "LEFT", "DOWN", "UP", "BUTTON_ONE", "BUTTON_TWO"]
                .into_iter()
                .enumerate()
            {
                // ...
                scopes.last_mut().unwrap().insert(
                    Path(vec![Name(String::from("Input")), Name(String::from(field))]),
                    Binding::Value(false, Value::Num(Num(offset as isize))),
                );
            }
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![Name(String::from("Array")), Name(String::from("get"))]),
                Binding::Intrinsic(Intrinsic::ArrayGet),
            );
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![Name(String::from("Array")), Name(String::from("set"))]),
                Binding::Intrinsic(Intrinsic::ArraySet),
            );
            // ...
            scopes.last_mut().unwrap().insert(
                Path(vec![
                    Name(String::from("String")),
                    Name(String::from("char_code")),
                ]),
                Binding::Intrinsic(Intrinsic::CharCode),
            );
        }
    }
}

impl<E: Environ> Backend<E> {
    pub fn render(&mut self, input: InputState) {
        // ...
        let Some(scopes) = &mut self.scopes else {
            return;
        };
        // ...
        let Some(Binding::Func(render)) =
            Self::bind_ref(scopes, &Path(vec![Name(String::from("render"))])).cloned()
        else {
            return;
        };
        // ...
        let Some((window_arg, window_path)) = render.2.get(0).cloned() else {
            return;
        };
        // ...
        let Some((input_arg, input_path)) = render.2.get(1).cloned() else {
            return;
        };
        // ...
        if (window_path != Path(vec![Name(String::from("Window"))]))
            || (input_path != Path(vec![Name(String::from("Input"))]))
        {
            return;
        }
        // ...
        let mut scope = HashMap::new();
        // ...
        if window_arg != Name(String::from("_")) {
            scope.insert(Path(vec![window_arg]), Binding::Value(false, Value::Window));
        }
        // ...
        if input_arg != Name(String::from("_")) {
            scope.insert(Path(vec![input_arg]), Binding::Value(false, Value::Input));
        }
        // ...
        scopes.push(scope);
        // ...
        Self::eval_body(&mut self.environ, scopes, &render.3, input).expect("RUNTIME ERROR!");
        // ...
        scopes.pop();
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn bind_ref<'s>(scopes: &'s Vec<Scope>, path: &Path<Owned>) -> Option<&'s Binding> {
        // ...
        scopes.iter().rev().find_map(|scope| {
            // ...
            scope.get(path)
        })
    }

    /// ...
    fn bind_mut<'s>(scopes: &'s mut Vec<Scope>, path: &Path<Owned>) -> Option<&'s mut Binding> {
        // ...
        scopes.iter_mut().rev().find_map(|scope| {
            // ...
            scope.get_mut(path)
        })
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn eval_item(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        item: &Item<Owned>,
        input: InputState,
    ) -> Option<()> {
        // ...
        match item {
            Item::Bind(_, bind) => Self::eval_bind(environ, scopes, bind, input),
            Item::Func(_, func) => Self::eval_func(scopes, func),
        }
    }

    /// ...
    fn eval_bind(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        bind: &Bind<Owned>,
        input: InputState,
    ) -> Option<()> {
        // ...
        match bind {
            // ...
            Bind::Let(name, _, expr) => {
                // ...
                let value = Self::eval_expr(environ, scopes, expr, input)?;
                // ...
                scopes
                    .last_mut()?
                    .insert(Path(vec![name.clone()]), Binding::Value(false, value));
                // ...
                Some(())
            }
            // ...
            Bind::Var(name, _, expr) => {
                // ...
                let value = Self::eval_expr(environ, scopes, expr, input)?;
                // ...
                scopes
                    .last_mut()?
                    .insert(Path(vec![name.clone()]), Binding::Value(true, value));
                // ...
                Some(())
            }
        }
    }

    /// ...
    fn eval_assn(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        assn: &Assn<Owned>,
        input: InputState,
    ) -> Option<()> {
        // ...
        let value = Self::eval_expr(environ, scopes, &assn.1, input)?;
        // ...
        let item = Self::bind_mut(scopes, &Path(vec![assn.0.clone()]))?;
        // ...
        *item = Binding::Value(true, value);
        // ...
        Some(())
    }

    /// ...
    fn eval_func(scopes: &mut Vec<Scope>, func: &Func<Owned>) -> Option<()> {
        // ...
        scopes
            .last_mut()?
            .insert(Path(vec![func.0.clone()]), func.clone().into());
        // ...
        Some(())
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn eval_stmt(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        stmt: &Stmt<Owned>,
        input: InputState,
    ) -> Option<Option<Value>> {
        // ...
        match stmt {
            Stmt::Bind(bind) => Self::eval_bind(environ, scopes, bind, input).map(|_| None),
            Stmt::Assn(assn) => Self::eval_assn(environ, scopes, assn, input).map(|_| None),
            Stmt::Func(func) => Self::eval_func(scopes, func).map(|_| None),
            Stmt::Call(call) => Self::eval_call(environ, scopes, call, input).map(|_| None),
            Stmt::Cond(cond) => Self::eval_cond(environ, scopes, cond, input),
            Stmt::Loop(fold) => Self::eval_fold(environ, scopes, fold, input),
            Stmt::Ret(expr) => expr.as_ref().map_or(Some(Some(Value::Unit)), |expr| {
                Self::eval_expr(environ, scopes, expr, input).map(Some)
            }),
        }
    }

    /// ...
    fn eval_cond(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        cond: &Cond<Owned>,
        input: InputState,
    ) -> Option<Option<Value>> {
        // ...
        let Value::Bool(pred) = Self::eval_expr(environ, scopes, &cond.0, input)? else {
            // ...
            return None;
        };
        // ...
        if pred {
            // ...
            return match Self::eval_body_inner(environ, scopes, &cond.1, input) {
                ControlFlow::Continue(_) => Some(None),
                ControlFlow::Break(value) => value.map(Some),
            };
        }
        // ...
        Some(None)
    }

    /// ...
    fn eval_fold(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        fold: &Loop<Owned>,
        input: InputState,
    ) -> Option<Option<Value>> {
        // ...
        loop {
            // ...
            let Value::Bool(pred) = Self::eval_expr(environ, scopes, &fold.0, input)? else {
                // ...
                return None;
            };
            // ...
            if pred {
                // ...
                match Self::eval_body_inner(environ, scopes, &fold.1, input) {
                    ControlFlow::Continue(_) => {}
                    ControlFlow::Break(value) => return value.map(Some),
                }
            } else {
                // ...
                return Some(None);
            }
        }
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn eval_body_inner<'s>(
        environ: &mut E,
        scopes: &'s mut Vec<Scope>,
        body: &Body<Owned>,
        input: InputState,
    ) -> ControlFlow<Option<Value>, &'s mut Vec<HashMap<Path<String>, Binding>>> {
        // ...
        body.0.iter().try_fold(scopes, |scopes, stmt| {
            // ...
            Self::eval_stmt(environ, scopes, stmt, input).map_or(
                // Break on invalid statement.
                ControlFlow::Break(None),
                // Evaluate the statement return result.
                |result| {
                    result.map_or(
                        // Continue without breaking.
                        ControlFlow::Continue(scopes),
                        // Break on early return.
                        |value| ControlFlow::Break(Some(value)),
                    )
                },
            )
        })
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn eval_path(scopes: &Vec<Scope>, path: &Path<Owned>) -> Option<Value> {
        // ...
        Self::bind_ref(scopes, path).map_or(None, |binding| match binding {
            Binding::Value(_, value) => Some(value.clone()),
            _ => panic!(
                "Variable `{}` is undefined",
                path.0
                    .iter()
                    .map(|name| name.0.clone())
                    .reduce(|path, name| format!("{}.{}", path, name))
                    .unwrap()
            ),
        })
    }

    /// ...
    fn eval_body(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        body: &Body<Owned>,
        input: InputState,
    ) -> Option<Value> {
        // ...
        match Self::eval_body_inner(environ, scopes, body, input) {
            // Default to returning `unit` value.
            ControlFlow::Continue(_) => Some(Value::Unit),
            // Return with error or value.
            ControlFlow::Break(value) => value,
        }
    }

    /// ...
    fn eval_call(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        call: &Call<Owned>,
        input: InputState,
    ) -> Option<Value> {
        // println!("CALL: {:?}, {:?}", scopes, call);
        // ...
        match Self::bind_ref(scopes, &call.0)?.clone() {
            // ...
            Binding::Func(func) => {
                // ...
                let (scopes, scope) = func.2.iter().zip(call.1.iter()).try_fold(
                    (scopes, HashMap::new()),
                    |(scopes, mut scope), ((name, _), expr)| {
                        // ...
                        let value = Self::eval_expr(environ, scopes, expr, input)?;
                        // ...
                        scope.insert(Path(vec![name.clone()]), Binding::Value(false, value));
                        // ...
                        Some((scopes, scope))
                    },
                )?;
                // ...
                scopes.push(scope);
                // ...
                let result = Self::eval_body(environ, scopes, &func.3, input)?;
                // ...
                scopes.pop();
                // ...
                Some(result)
            }
            // ...
            Binding::Intrinsic(Intrinsic::Draw) => {
                // ...
                let Value::Window = Self::eval_expr(environ, scopes, call.1.get(0)?, input)? else {
                    return None;
                };
                // ...
                let Value::Num(tile) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                let Value::Num(x) = Self::eval_expr(environ, scopes, call.1.get(2)?, input)? else {
                    return None;
                };
                // ...
                let Value::Num(y) = Self::eval_expr(environ, scopes, call.1.get(3)?, input)? else {
                    return None;
                };
                // ...
                environ.draw_tile(tile.0 as _, x.0 as _, y.0 as _);
                // ...
                Some(Value::Unit)
            }
            // ...
            Binding::Intrinsic(Intrinsic::Pressed) => {
                // ...
                let Value::Input = Self::eval_expr(environ, scopes, call.1.get(0)?, input)? else {
                    return None;
                };
                // ...
                let Value::Num(button) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                if button.0 < 0 || button.0 > 5 {
                    return None;
                }
                // ...
                return Some(Value::Bool((input.0 & (1 << button.0)) != 0));
            }
            // ...
            Binding::Intrinsic(Intrinsic::ArrayGet) => {
                // ...
                let Value::Arr(arr) = Self::eval_expr(environ, scopes, call.1.get(0)?, input)?
                else {
                    return None;
                };
                // ...
                let Value::Num(idx) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                (usize::try_from(idx.0))
                    .ok()
                    .map(|idx| {
                        arr.get(idx).expect(&format!(
                            "Array.get() OOB: len = {}, idx = {}",
                            arr.len(),
                            idx
                        ))
                    })
                    .cloned()
            }
            // ...
            Binding::Intrinsic(Intrinsic::ArraySet) => {
                // ...
                let Value::Arr(mut arr) = Self::eval_expr(environ, scopes, call.1.get(0)?, input)?
                else {
                    return None;
                };
                // ...
                let Value::Num(idx) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                let val = Self::eval_expr(environ, scopes, call.1.get(2)?, input)?;
                // ...
                let len = arr.len();
                // ...
                (usize::try_from(idx.0)).ok().map(|idx| {
                    *(arr
                        .get_mut(idx)
                        .expect(&format!("Array.set() OOB: len = {}, idx = {}", len, idx))) = val;
                    Value::Arr(arr)
                })
            }
            // ...
            Binding::Intrinsic(Intrinsic::CharCode) => {
                // ...
                let Value::Str(str) = Self::eval_expr(environ, scopes, call.1.get(0)?, input)?
                else {
                    return None;
                };
                // ...
                let Value::Num(idx) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                let idx: usize = idx.0.try_into().expect(&format!(
                    "String.char_code() invalid index: idx = {}",
                    idx.0
                ));
                // ...
                let chr: u8 = str.0.as_bytes().get(idx).copied().expect(&format!(
                    "String.char_code() invalid ASCII: str = {}, idx = {}",
                    str.0, idx
                ));
                // ...
                Some(Value::Num(Num(chr as isize)))
            }
            // ...
            Binding::Intrinsic(Intrinsic::LoadTilemap) => {
                // ...
                let Value::Window = Self::eval_expr(environ, scopes, call.1.get(0)?, input)? else {
                    return None;
                };
                // ...
                let Value::Str(str) = Self::eval_expr(environ, scopes, call.1.get(1)?, input)?
                else {
                    return None;
                };
                // ...
                environ.load_tile(&str.0);
                // ...
                Some(Value::Unit)
            }
            // ...
            Binding::Value(_, _) => None,
        }
    }

    /// ...
    fn eval_prim(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        prim: &Prim<Owned>,
        input: InputState,
    ) -> Option<Value> {
        // println!("PRIM: {:?}, {:?}", scopes, prim);
        // ...
        match prim {
            Prim::Arr(arr) => Some(Value::Arr(
                (0..arr.0).into_iter().map(|_| Value::Num(Num(0))).collect(),
            )),
            Prim::Num(num) => Some(Value::Num(num.clone())),
            Prim::Str(str) => Some(Value::Str(str.clone())),
            Prim::Bool(bool) => Some(Value::Bool(*bool)),
            Prim::Path(path) => Self::eval_path(scopes, path),
            Prim::Call(call) => Self::eval_call(environ, scopes, call, input),
            Prim::Sect(expr) => Self::eval_expr(environ, scopes, expr, input),
        }
    }

    /// ...
    fn eval_expr(
        environ: &mut E,
        scopes: &mut Vec<Scope>,
        expr: &Expr<Owned>,
        input: InputState,
    ) -> Option<Value> {
        // println!("EXPR: {:?}, {:?}", scopes, expr);
        // ...
        match expr {
            Expr::Prim(prim) => Self::eval_prim(environ, scopes, prim, input),
            Expr::Comp(comp) => match Self::eval_prim(environ, scopes, comp, input) {
                Some(Value::Bool(value)) => Some(Value::Bool(!value)),
                _ => None,
            },
            Expr::And(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Bool(lhs)), Some(Value::Bool(rhs))) => {
                        Some(Value::Bool(lhs && rhs))
                    }
                    _ => None,
                }
            }
            Expr::Orr(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Bool(lhs)), Some(Value::Bool(rhs))) => {
                        Some(Value::Bool(lhs || rhs))
                    }
                    _ => None,
                }
            }
            Expr::Add(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        Some(Value::Num((lhs.0 + rhs.0).into()))
                    }
                    _ => None,
                }
            }
            Expr::Sub(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        Some(Value::Num((lhs.0 - rhs.0).into()))
                    }
                    _ => None,
                }
            }
            Expr::Mul(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        Some(Value::Num((lhs.0 * rhs.0).into()))
                    }
                    _ => None,
                }
            }
            Expr::Div(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        Some(Value::Num((lhs.0 / rhs.0).into()))
                    }
                    _ => None,
                }
            }
            Expr::Mod(lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        Some(Value::Num((lhs.0 % rhs.0).into()))
                    }
                    _ => None,
                }
            }
            Expr::BitNot(val) => match Self::eval_prim(environ, scopes, val, input) {
                Some(Value::Num(val)) => Some(Value::Num(Num(!val.0))),
                _ => None,
            },
            Expr::BitShl(lhs, rhs) => match (
                Self::eval_prim(environ, scopes, lhs, input),
                Self::eval_prim(environ, scopes, rhs, input),
            ) {
                (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                    Some(Value::Num((lhs.0 << rhs.0).into()))
                }
                _ => None,
            },
            Expr::BitAnd(lhs, rhs) => match (
                Self::eval_prim(environ, scopes, lhs, input),
                Self::eval_prim(environ, scopes, rhs, input),
            ) {
                (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                    Some(Value::Num((lhs.0 & rhs.0).into()))
                }
                _ => None,
            },
            Expr::BitOrr(lhs, rhs) => match (
                Self::eval_prim(environ, scopes, lhs, input),
                Self::eval_prim(environ, scopes, rhs, input),
            ) {
                (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                    Some(Value::Num((lhs.0 | rhs.0).into()))
                }
                _ => None,
            },
            Expr::Ord(comp, ordering, lhs, rhs) => {
                match (
                    Self::eval_prim(environ, scopes, lhs, input),
                    Self::eval_prim(environ, scopes, rhs, input),
                ) {
                    (Some(Value::Num(lhs)), Some(Value::Num(rhs))) => {
                        let value = match ordering {
                            Ordering::Less => Some(lhs.0 < rhs.0),
                            Ordering::Equal => Some(lhs.0 == rhs.0),
                            Ordering::Greater => Some(lhs.0 > rhs.0),
                        }?;
                        Some(if *comp {
                            Value::Bool(!value)
                        } else {
                            Value::Bool(value)
                        })
                    }
                    _ => None,
                }
            }
        }
    }
}
