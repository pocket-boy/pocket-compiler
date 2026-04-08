use std::ops::ControlFlow;

use im::HashMap;
use nom_supreme::{error::ErrorTree, final_parser::final_parser};

use crate::{
    compiler::Compiler,
    input::InputState,
    parser::{Assn, Bind, Body, Call, Cond, Expr, File, Func, Item, Name, Owned, Parser, Stmt},
};

/// ...
pub struct Backend<E> {
    /// ...
    pub compiler: Compiler,
    /// ...
    pub environ: E,
    /// ...
    pub result: Option<File<Owned>>,
    /// ...
    pub bindings: Option<HashMap<Name<Owned>, Binding>>,
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
        let result = compiler
            .modules
            .get("main")
            .and_then(|module| dbg!(parser(module)).map(File::into).ok());
        // ...
        let mut instance = Self {
            compiler: compiler.clone(),
            environ,
            result,
            bindings: None,
        };
        // ...
        instance.prepare();
        // ...
        instance
    }

    /// ...
    pub fn prepare(&mut self) {
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
                        map.insert(
                            name.clone(),
                            Binding::Value(self.eval_expr(&map, expr, InputState(0))?),
                        );
                        Some(map)
                    }
                    // ...
                    Bind::Var(name, _, expr) => {
                        map.insert(
                            name.clone(),
                            Binding::Value(self.eval_expr(&map, expr, InputState(0))?),
                        );
                        Some(map)
                    }
                },
                // ...
                Stmt::Func(func @ Func(name, _, _, _)) => {
                    map.insert(name.clone(), Binding::Func(func.clone()));
                    Some(map)
                }
                // ...
                _ => Some(map),
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
        bindings.insert(
            render.2.first().unwrap().0.clone(),
            Binding::Value(Value::Window),
        );
        bindings.insert(
            render.2.get(1).unwrap().0.clone(),
            Binding::Value(Value::Input),
        );
        // ...
        self.bindings = Some(bindings);
    }

    /// ...
    pub fn render(&mut self, input: InputState) {
        // ...
        let Some(bindings) = self.bindings.clone() else {
            return;
        };
        // ...
        let Some(Binding::Func(render)) = bindings.get(&Name("render".into())).cloned() else {
            return;
        };
        // ...
        self.eval_body(&bindings, &render.3, input);
    }
}

impl<E: Environ> Backend<E> {
    // ...
    fn eval_body(
        &mut self,
        map: &HashMap<Name<Owned>, Binding>,
        body: &Body<Owned>,
        input: InputState,
    ) -> Option<Value> {
        // ...
        match body.0.iter().try_fold(map.clone(), |mut map, stmt| {
            // ...
            match stmt {
                // ...
                Stmt::Bind(bind) => match bind {
                    // ...
                    Bind::Let(name, _, expr) => self
                        .eval_expr(&map, expr, input)
                        .map(|value| {
                            map.insert(name.clone(), Binding::Value(value));
                            ControlFlow::Continue(map)
                        })
                        .unwrap_or(ControlFlow::Break(None)),
                    // ...
                    Bind::Var(name, _, expr) => self
                        .eval_expr(&map, expr, input)
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
                Stmt::Cond(Cond(cond, body)) => match self.eval_expr(&map, cond, input) {
                    // ...
                    Some(Value::Bool(false)) => ControlFlow::Continue(map),
                    // ...
                    Some(Value::Bool(true)) => {
                        // ...
                        self.eval_body(&map, body, input)
                            .map_or(ControlFlow::Break(None), |_| ControlFlow::Continue(map))
                    }
                    _ => ControlFlow::Break(None),
                },
                // ...
                Stmt::Assn(Assn(name, expr)) => {
                    // ...
                    self.eval_expr(&map, expr, input)
                        .map_or(ControlFlow::Break(None), |value| {
                            map.insert(name.clone(), Binding::Value(value));
                            ControlFlow::Continue(map)
                        })
                }
                // ...
                Stmt::Call(call) => {
                    // ...
                    if call.0 == Name("draw".into()) {
                        return self.handle_draw(map, call, input);
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
                            args.insert(
                                name.clone(),
                                Binding::Value(self.eval_expr(&map, expr, input)?),
                            );
                            // ...
                            Some(args)
                        },
                    ) else {
                        return ControlFlow::Break(None);
                    };
                    // ...
                    self.eval_body(&args, &func.3, input);
                    // ...
                    ControlFlow::Continue(map)
                }
                // ...
                Stmt::Ret(expr) => self
                    .eval_expr(&map, expr, input)
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
        input: InputState,
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
                if call.0 == Name("left".into()) {
                    return self.handle_button(map, call, input, 0b00000010);
                }
                // ...
                if call.0 == Name("right".into()) {
                    return self.handle_button(map, call, input, 0b00000001);
                }
                // ...
                if call.0 == Name("up".into()) {
                    return self.handle_button(map, call, input, 0b00001000);
                }
                // ...
                if call.0 == Name("down".into()) {
                    return self.handle_button(map, call, input, 0b00000100);
                }
                // ...
                if call.0 == Name("button_one".into()) {
                    return self.handle_button(map, call, input, 0b00100000);
                }
                // ...
                if call.0 == Name("button_two".into()) {
                    return self.handle_button(map, call, input, 0b00010000);
                }
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
                        args.insert(
                            name.clone(),
                            Binding::Value(self.eval_expr(map, expr, input)?),
                        );
                        // ...
                        Some(args)
                    },
                )?;
                // ...
                self.eval_body(&args, &func.3, input)
            }
        }
    }
}

impl<E: Environ> Backend<E> {
    /// ...
    fn handle_draw(
        &mut self,
        map: HashMap<Name<Owned>, Binding>,
        call: &Call<Owned>,
        input: InputState,
    ) -> ControlFlow<Option<Value>, HashMap<Name<Owned>, Binding>> {
        // ...
        if call
            .1
            .first()
            .and_then(|window| self.eval_expr(&map, window, input))
            != Some(Value::Window)
        {
            return ControlFlow::Break(None);
        }
        // ...
        let Some(Value::Num(tile)) = call
            .1
            .get(1)
            .and_then(|tile| self.eval_expr(&map, tile, input))
        else {
            return ControlFlow::Break(None);
        };
        // ...
        let Some(Value::Num(x)) = call.1.get(2).and_then(|x| self.eval_expr(&map, x, input)) else {
            return ControlFlow::Break(None);
        };
        // ...
        let Some(Value::Num(y)) = call.1.get(3).and_then(|y| self.eval_expr(&map, y, input)) else {
            return ControlFlow::Break(None);
        };
        // ...
        self.environ.draw_tile(tile as u32, x as u32, y as u32);
        // ...
        ControlFlow::Continue(map)
    }

    /// ...
    fn handle_button(
        &mut self,
        map: &HashMap<Name<Owned>, Binding>,
        call: &Call<Owned>,
        input: InputState,
        mask: u8,
    ) -> Option<Value> {
        // ...
        if call
            .1
            .first()
            .and_then(|value| self.eval_expr(map, value, input))
            != Some(Value::Input)
        {
            return None;
        }
        // ...
        if (input.0 & mask) != 0 {
            return Some(Value::Bool(true));
        }
        Some(Value::Bool(false))
    }
}
