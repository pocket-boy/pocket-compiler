use std::ops::ControlFlow;

use im::HashMap;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::{
    compiler::Compiler,
    parser::{Bind, Expr, File, Func, Item, Name, Owned, Parser, Stmt},
};

/// ...
pub struct Backend<E> {
    /// ...
    pub compiler: Compiler,
    /// ...
    pub environ: E,
    /// ...
    pub result: Option<File<Owned>>,
}

/// ...
pub trait Environ {
    /// ...
    fn action_one(&mut self, value: u32);
    /// ...
    fn action_two(&mut self, value: u32);
    /// ...
    fn draw_tile(&mut self, tile: u32, x: u32, y: u32);
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Binding {
    /// ...
    Value(Value),
    /// ...
    Func(Func<Owned>),
}

/// ...
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// ...
    Num(usize),
    /// ...
    Str(String),
    /// ...
    Bool(bool),
    /// ...
    Unit,
    /// ...
    Window,
    /// ...
    Input,
}

impl<E: Environ> Backend<E> {
    /// ...
    pub fn new(compiler: Compiler, environ: E) -> Self {
        // ...
        let mut parser = final_parser::<_, _, _, ErrorTree<&str>>(Parser::file);
        // ...
        Self {
            compiler: compiler.clone(),
            environ,
            result: compiler
                .modules
                .get("main")
                .and_then(|module| parser(module).map(File::into).ok()),
        }
    }

    /// ...
    pub fn render(&mut self) {
        // ...
        let Some(file) = self.result.clone() else {
            return;
        };
        // ...
        let Some(mut bindings) = file.0.iter().try_fold(
            // ...
            HashMap::<Name<Owned>, Binding>::new(),
            // ...
            |mut map, Item(_, stmt)| match stmt {
                // ...
                Stmt::Bind(bind) => match bind {
                    // ...
                    Bind::Let(name, _, expr) => {
                        map.insert(name.clone(), Binding::Value(self.eval_expr(&map, expr)?));
                        Some(map)
                    }
                    // ...
                    Bind::Var(name, _, expr) => {
                        map.insert(name.clone(), Binding::Value(self.eval_expr(&map, expr)?));
                        Some(map)
                    }
                },
                // ...
                Stmt::Func(func @ Func(name, _, _, _)) => {
                    map.insert(name.clone(), Binding::Func(func.clone()));
                    Some(map)
                }
                // ...
                Stmt::Call(_) => Some(map),
                // ...
                Stmt::Ret(_) => Some(map),
            },
        ) else {
            return;
        };
        // ...
        let Some(Binding::Func(render)) = bindings.get(&Name("render".into())).cloned() else {
            return;
        };
        // ...
        if Some(&Name("Window".into())) != render.2.first().map(|(_, anno)| anno)
            || Some(&Name("Input".into())) != render.2.get(1).map(|(_, anno)| anno)
        {
            return;
        }
        // ...
        println!("made to render");
        // ...
        bindings.insert(
            render.2.first().unwrap().0.clone(),
            Binding::Value(Value::Window),
        );
        bindings.insert(
            render.2.get(1).unwrap().0.clone(),
            Binding::Value(Value::Input),
        );
        // ...
        self.eval_func(&bindings, &render);
    }
}

impl<E: Environ> Backend<E> {
    // ...
    fn eval_func(
        &mut self,
        map: &HashMap<Name<Owned>, Binding>,
        func: &Func<Owned>,
    ) -> Option<Value> {
        // ...
        match func.3.0.iter().try_fold(map.clone(), |mut map, stmt| {
            // ...
            match stmt {
                // ...
                Stmt::Bind(bind) => match bind {
                    // ...
                    Bind::Let(name, _, expr) => self
                        .eval_expr(&map, expr)
                        .map(|value| {
                            map.insert(name.clone(), Binding::Value(value));
                            ControlFlow::Continue(map)
                        })
                        .unwrap_or(ControlFlow::Break(None)),
                    // ...
                    Bind::Var(name, _, expr) => self
                        .eval_expr(&map, expr)
                        .map(|value| {
                            map.insert(name.clone(), Binding::Value(value));
                            ControlFlow::Continue(map)
                        })
                        .unwrap_or(ControlFlow::Break(None)),
                },
                // ...
                Stmt::Func(func @ Func(name, _, _, _)) => {
                    map.insert(name.clone(), Binding::Func(func.clone()));
                    ControlFlow::Continue(map)
                }
                // ...
                Stmt::Call(call) => {
                    // ...
                    if call.0 == Name("draw".into()) {
                        // ...
                        if call
                            .1
                            .first()
                            .and_then(|window| self.eval_expr(&map, window))
                            != Some(Value::Window)
                        {
                            return ControlFlow::Break(None);
                        }
                        // ...
                        let Some(Value::Num(tile)) =
                            call.1.get(1).and_then(|tile| self.eval_expr(&map, tile))
                        else {
                            return ControlFlow::Break(None);
                        };
                        // ...
                        let Some(Value::Num(x)) =
                            call.1.get(2).and_then(|x| self.eval_expr(&map, x))
                        else {
                            return ControlFlow::Break(None);
                        };
                        // ...
                        let Some(Value::Num(y)) =
                            call.1.get(3).and_then(|y| self.eval_expr(&map, y))
                        else {
                            return ControlFlow::Break(None);
                        };
                        // ...
                        self.environ.draw_tile(tile as u32, x as u32, y as u32);
                    }
                    // ...
                    let Some(Binding::Func(func)) = map.get(&call.0) else {
                        return ControlFlow::Break(None);
                    };
                    // ...
                    let Some(args) = func.2.iter().zip(call.1.iter()).try_fold(
                        map.clone(),
                        |mut args, ((name, _), expr)| {
                            // ...
                            args.insert(name.clone(), Binding::Value(self.eval_expr(&map, expr)?));
                            // ...
                            Some(args)
                        },
                    ) else {
                        return ControlFlow::Break(None);
                    };
                    // ...
                    self.eval_func(&args, func);
                    // ...
                    ControlFlow::Continue(map)
                }
                // ...
                Stmt::Ret(expr) => self
                    .eval_expr(&map, expr)
                    .map(|value| ControlFlow::Break(Some(value)))
                    .unwrap_or(ControlFlow::Break(None)),
            }
        }) {
            ControlFlow::Continue(_) => Some(Value::Unit),
            ControlFlow::Break(result) => result,
        }
    }

    // ...
    fn eval_expr(
        &mut self,
        map: &HashMap<Name<Owned>, Binding>,
        expr: &Expr<Owned>,
    ) -> Option<Value> {
        match expr {
            Expr::Num(num) => Some(Value::Num(num.0)),
            Expr::Str(str) => Some(Value::Str(str.0.clone())),
            Expr::Bool(bool) => Some(Value::Bool(*bool)),
            Expr::Name(name) => map.get(name).and_then(|binding| match binding {
                Binding::Value(value) => Some(value.clone()),
                Binding::Func(_) => None,
            }),
            Expr::Call(call) => {
                // ...
                let Some(Binding::Func(func)) = map.get(&call.0) else {
                    println!("missing func: {:?}", call.0);
                    return None;
                };
                // ...
                let args = func.2.iter().zip(call.1.iter()).try_fold(
                    map.clone(),
                    |mut args, ((name, _), expr)| {
                        // ...
                        args.insert(name.clone(), Binding::Value(self.eval_expr(&map, expr)?));
                        // ...
                        Some(args)
                    },
                )?;
                // ...
                self.eval_func(&args, func)
            }
        }
    }
}
