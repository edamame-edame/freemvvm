//! Pure-Rust Lightweight JavaScript Engine VM & Sandboxed DOM / MVVM Bridge (`UNIT-WEBVIEW-007`, `UNIT-WEBVIEW-008`).

use std::collections::BTreeMap;

use crate::css::{parse_stylesheet, selector_matches};
use crate::dom::{DomArena, NodeId, ResourceBudget, WebViewError};

/// Runtime value inside the sandboxed JavaScript subset VM.
#[derive(Clone, Debug, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsValue>),
    Object(BTreeMap<String, JsValue>),
    DomElement(NodeId),
    DomClassList(NodeId),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

impl JsValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0.0 && !n.is_nan(),
            Self::Str(s) => !s.is_empty(),
            Self::Array(_)
            | Self::Object(_)
            | Self::DomElement(_)
            | Self::DomClassList(_)
            | Self::Function { .. } => true,
        }
    }

    pub fn to_js_string(&self) -> String {
        match self {
            Self::Undefined => "undefined".to_owned(),
            Self::Null => "null".to_owned(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", *n)
                } else {
                    n.to_string()
                }
            }
            Self::Str(s) => s.clone(),
            Self::Array(items) => {
                let parts: Vec<String> = items.iter().map(Self::to_js_string).collect();
                parts.join(",")
            }
            Self::Object(_) => "[object Object]".to_owned(),
            Self::DomElement(id) => format!("[object HTMLElement({})]", id.0),
            Self::DomClassList(id) => format!("[object DOMTokenList({})]", id.0),
            Self::Function { .. } => "[Function]".to_owned(),
        }
    }

    pub fn estimated_bytes(&self) -> usize {
        match self {
            Self::Undefined | Self::Null | Self::Bool(_) | Self::Number(_) => 16,
            Self::DomElement(_) | Self::DomClassList(_) => 16,
            Self::Str(s) => 24 + s.len(),
            Self::Array(items) => 24 + items.iter().map(Self::estimated_bytes).sum::<usize>(),
            Self::Object(map) => {
                32 + map
                    .iter()
                    .map(|(k, v)| 24 + k.len() + v.estimated_bytes())
                    .sum::<usize>()
            }
            Self::Function { params, body } => 64 + params.len() * 16 + body.len() * 32,
        }
    }
}

/// Host bridge trait connecting the sandboxed Script VM to FreeMVVM `Property` and `Command` bindings (`SD-WV-07`).
pub trait ScriptHostBridge {
    fn get_property(&self, alias: &str) -> Result<JsValue, WebViewError>;
    fn set_property(&mut self, alias: &str, value: JsValue) -> Result<(), WebViewError>;
    fn invoke_command(&mut self, alias: &str) -> Result<bool, WebViewError>;
}

/// Default in-memory host bridge for standalone WebView and unit testing.
#[derive(Clone, Debug, Default)]
pub struct InMemoryHostBridge {
    properties: BTreeMap<String, (JsValue, bool)>,
    commands_enabled: BTreeMap<String, bool>,
    invoked_commands: Vec<String>,
}

impl InMemoryHostBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expose_property(&mut self, alias: impl Into<String>, initial: JsValue, writable: bool) {
        self.properties.insert(alias.into(), (initial, writable));
    }

    pub fn expose_command(&mut self, alias: impl Into<String>, can_execute: bool) {
        self.commands_enabled.insert(alias.into(), can_execute);
    }

    pub fn invoked_commands(&self) -> &[String] {
        &self.invoked_commands
    }
}

impl ScriptHostBridge for InMemoryHostBridge {
    fn get_property(&self, alias: &str) -> Result<JsValue, WebViewError> {
        self.properties
            .get(alias)
            .map(|(v, _)| v.clone())
            .ok_or_else(|| WebViewError::UnexposedProperty(alias.to_owned()))
    }

    fn set_property(&mut self, alias: &str, value: JsValue) -> Result<(), WebViewError> {
        let Some((slot, writable)) = self.properties.get_mut(alias) else {
            return Err(WebViewError::UnexposedProperty(alias.to_owned()));
        };
        if !*writable {
            return Err(WebViewError::ReadOnlyProperty(alias.to_owned()));
        }
        *slot = value;
        Ok(())
    }

    fn invoke_command(&mut self, alias: &str) -> Result<bool, WebViewError> {
        let Some(&can_execute) = self.commands_enabled.get(alias) else {
            return Err(WebViewError::UnexposedCommand(alias.to_owned()));
        };
        if can_execute {
            self.invoked_commands.push(alias.to_owned());
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Let,
    Const,
    Var,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    True,
    False,
    Null,
    Undefined,
    Ident(String),
    Number(f64),
    Str(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semi,
    Comma,
    Dot,
    Colon,
    Assign,
    PlusAssign,
    MinusAssign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqStrict,
    NeStrict,
    EqLoose,
    NeLoose,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(JsValue),
    Var(String),
    ArrayLit(Vec<Expr>),
    ObjectLit(Vec<(String, Expr)>),
    FunctionExpr {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    UnaryNot(Box<Expr>),
    UnaryNeg(Box<Expr>),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    VarDecl {
        name: String,
        init: Expr,
    },
    Assign {
        name: String,
        expr: Expr,
    },
    MemberAssign {
        object: Expr,
        property: String,
        expr: Expr,
    },
    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    ExprStmt(Expr),
}

/// Registered DOM event listener inside `ScriptVm`.
#[derive(Clone, Debug, PartialEq)]
pub struct DomEventListener {
    pub node_id: NodeId,
    pub event_type: String,
    pub callback: JsValue,
}

/// Sandboxed lightweight JavaScript engine VM (`UNIT-WEBVIEW-007`, `UNIT-WEBVIEW-008`).
#[derive(Clone, Debug)]
pub struct ScriptVm {
    budget: ResourceBudget,
    globals: BTreeMap<String, JsValue>,
    listeners: Vec<DomEventListener>,
    fuel_remaining: usize,
    call_depth: usize,
}

impl ScriptVm {
    pub fn new(budget: ResourceBudget) -> Self {
        Self {
            budget,
            globals: BTreeMap::new(),
            listeners: Vec::new(),
            fuel_remaining: budget.max_script_steps,
            call_depth: 0,
        }
    }

    pub fn globals(&self) -> &BTreeMap<String, JsValue> {
        &self.globals
    }

    pub fn listeners(&self) -> &[DomEventListener] {
        &self.listeners
    }

    /// Evaluates a script string transactionally against `dom` and `host`.
    /// If execution fails or exceeds fuel/stack/heap limits, `dom` and VM globals/listeners are rolled back (`SD-WV-05`).
    pub fn eval_script(
        &mut self,
        source: &str,
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<JsValue, WebViewError> {
        if source.len() > self.budget.max_source_bytes {
            return Err(WebViewError::SourceBytesLimitExceeded {
                actual: source.len(),
                max: self.budget.max_source_bytes,
            });
        }
        let stmts = parse_program(source)?;
        let dom_backup = dom.clone();
        let globals_backup = self.globals.clone();
        let listeners_backup = self.listeners.clone();

        self.fuel_remaining = self.budget.max_script_steps;
        self.call_depth = 0;

        let mut local_scopes: Vec<BTreeMap<String, JsValue>> = Vec::new();
        match self.exec_block(&stmts, &mut local_scopes, dom, host) {
            Ok((ret, last_val)) => Ok(ret.unwrap_or(last_val)),
            Err(err) => {
                *dom = dom_backup;
                self.globals = globals_backup;
                self.listeners = listeners_backup;
                Err(err)
            }
        }
    }

    /// Dispatches a DOM event (e.g. `"click"`, `"input"`) on `target_node` to any registered listeners.
    pub fn dispatch_dom_event(
        &mut self,
        target_node: NodeId,
        event_type: &str,
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<(), WebViewError> {
        let matching: Vec<JsValue> = self
            .listeners
            .iter()
            .filter(|l| l.node_id == target_node && l.event_type == event_type)
            .map(|l| l.callback.clone())
            .collect();

        if matching.is_empty() {
            return Ok(());
        }

        let dom_backup = dom.clone();
        let globals_backup = self.globals.clone();
        let listeners_backup = self.listeners.clone();

        self.fuel_remaining = self.budget.max_script_steps;
        self.call_depth = 0;

        for cb in matching {
            let res = match cb {
                JsValue::Function { params, body } => self.call_user_function(
                    &params,
                    &body,
                    &[JsValue::DomElement(target_node)],
                    dom,
                    host,
                ),
                JsValue::Str(func_name) => {
                    if let Some(JsValue::Function { params, body }) =
                        self.globals.get(&func_name).cloned()
                    {
                        self.call_user_function(
                            &params,
                            &body,
                            &[JsValue::DomElement(target_node)],
                            dom,
                            host,
                        )
                    } else {
                        Err(WebViewError::ScriptRuntimeError(format!(
                            "listener function '{func_name}' not found"
                        )))
                    }
                }
                _ => Err(WebViewError::ScriptRuntimeError(
                    "invalid event listener callback".to_owned(),
                )),
            };
            if let Err(err) = res {
                *dom = dom_backup;
                self.globals = globals_backup;
                self.listeners = listeners_backup;
                return Err(err);
            }
        }
        Ok(())
    }

    fn consume_step(
        &mut self,
        local_scopes: &[BTreeMap<String, JsValue>],
    ) -> Result<(), WebViewError> {
        if self.fuel_remaining == 0 {
            return Err(WebViewError::ScriptStepLimitExceeded {
                max_steps: self.budget.max_script_steps,
            });
        }
        self.fuel_remaining -= 1;

        let global_bytes: usize = self
            .globals
            .iter()
            .map(|(k, v)| 24 + k.len() + v.estimated_bytes())
            .sum();
        let local_bytes: usize = local_scopes
            .iter()
            .flat_map(|s| s.iter())
            .map(|(k, v)| 24 + k.len() + v.estimated_bytes())
            .sum();
        if global_bytes + local_bytes > self.budget.max_heap_bytes {
            return Err(WebViewError::ScriptHeapLimitExceeded {
                max_bytes: self.budget.max_heap_bytes,
            });
        }
        Ok(())
    }

    fn exec_block(
        &mut self,
        stmts: &[Stmt],
        scopes: &mut Vec<BTreeMap<String, JsValue>>,
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<(Option<JsValue>, JsValue), WebViewError> {
        let mut last = JsValue::Undefined;
        for stmt in stmts {
            if let Some(ret) = self.exec_stmt(stmt, scopes, dom, host, &mut last)? {
                return Ok((Some(ret), last));
            }
        }
        Ok((None, last))
    }

    fn exec_stmt(
        &mut self,
        stmt: &Stmt,
        scopes: &mut Vec<BTreeMap<String, JsValue>>,
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
        last_val: &mut JsValue,
    ) -> Result<Option<JsValue>, WebViewError> {
        self.consume_step(scopes)?;
        match stmt {
            Stmt::VarDecl { name, init } => {
                let val = self.eval_expr(init, scopes, dom, host)?;
                if let Some(top) = scopes.last_mut() {
                    top.insert(name.clone(), val);
                } else {
                    self.globals.insert(name.clone(), val);
                }
                Ok(None)
            }
            Stmt::Assign { name, expr } => {
                let val = self.eval_expr(expr, scopes, dom, host)?;
                for scope in scopes.iter_mut().rev() {
                    if scope.contains_key(name) {
                        scope.insert(name.clone(), val);
                        return Ok(None);
                    }
                }
                self.globals.insert(name.clone(), val);
                Ok(None)
            }
            Stmt::MemberAssign {
                object,
                property,
                expr,
            } => {
                let target = self.eval_expr(object, scopes, dom, host)?;
                let val = self.eval_expr(expr, scopes, dom, host)?;
                match target {
                    JsValue::DomElement(node_id) => {
                        match property.as_str() {
                            "textContent" | "innerText" => {
                                dom.set_text_content(node_id, val.to_js_string())?;
                            }
                            "value" => {
                                dom.set_attribute(node_id, "value", val.to_js_string())?;
                            }
                            "id" | "className" => {
                                let attr = if property == "className" {
                                    "class"
                                } else {
                                    "id"
                                };
                                dom.set_attribute(node_id, attr, val.to_js_string())?;
                            }
                            _ => {
                                dom.set_attribute(node_id, property, val.to_js_string())?;
                            }
                        }
                        Ok(None)
                    }
                    _ => {
                        if let Expr::Var(var_name) = object {
                            self.assign_object_member(var_name, property, val, scopes);
                        }
                        Ok(None)
                    }
                }
            }
            Stmt::FuncDecl { name, params, body } => {
                let fn_val = JsValue::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                if let Some(top) = scopes.last_mut() {
                    top.insert(name.clone(), fn_val);
                } else {
                    self.globals.insert(name.clone(), fn_val);
                }
                Ok(None)
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let c = self.eval_expr(cond, scopes, dom, host)?;
                if c.is_truthy() {
                    let (ret, _) = self.exec_block(then_branch, scopes, dom, host)?;
                    Ok(ret)
                } else {
                    let (ret, _) = self.exec_block(else_branch, scopes, dom, host)?;
                    Ok(ret)
                }
            }
            Stmt::While { cond, body } => {
                loop {
                    self.consume_step(scopes)?;
                    let c = self.eval_expr(cond, scopes, dom, host)?;
                    if !c.is_truthy() {
                        break;
                    }
                    let (ret, _) = self.exec_block(body, scopes, dom, host)?;
                    if ret.is_some() {
                        return Ok(ret);
                    }
                }
                Ok(None)
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.exec_stmt(init_stmt, scopes, dom, host, last_val)?;
                }
                loop {
                    self.consume_step(scopes)?;
                    if let Some(c_expr) = cond {
                        let c = self.eval_expr(c_expr, scopes, dom, host)?;
                        if !c.is_truthy() {
                            break;
                        }
                    }
                    let (ret, _) = self.exec_block(body, scopes, dom, host)?;
                    if ret.is_some() {
                        return Ok(ret);
                    }
                    if let Some(step_stmt) = step {
                        self.exec_stmt(step_stmt, scopes, dom, host, last_val)?;
                    }
                }
                Ok(None)
            }
            Stmt::Return(expr_opt) => {
                let val = match expr_opt {
                    Some(e) => self.eval_expr(e, scopes, dom, host)?,
                    None => JsValue::Undefined,
                };
                Ok(Some(val))
            }
            Stmt::ExprStmt(expr) => {
                *last_val = self.eval_expr(expr, scopes, dom, host)?;
                Ok(None)
            }
        }
    }

    fn assign_object_member(
        &mut self,
        var_name: &str,
        property: &str,
        val: JsValue,
        scopes: &mut [BTreeMap<String, JsValue>],
    ) {
        for scope in scopes.iter_mut().rev() {
            if let Some(JsValue::Object(map)) = scope.get_mut(var_name) {
                map.insert(property.to_owned(), val);
                return;
            }
        }
        if let Some(JsValue::Object(map)) = self.globals.get_mut(var_name) {
            map.insert(property.to_owned(), val);
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        scopes: &mut Vec<BTreeMap<String, JsValue>>,
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<JsValue, WebViewError> {
        self.consume_step(scopes)?;
        match expr {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Var(name) => {
                for scope in scopes.iter().rev() {
                    if let Some(v) = scope.get(name) {
                        return Ok(v.clone());
                    }
                }
                if let Some(v) = self.globals.get(name) {
                    return Ok(v.clone());
                }
                Ok(JsValue::Undefined)
            }
            Expr::ArrayLit(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.eval_expr(item, scopes, dom, host)?);
                }
                Ok(JsValue::Array(out))
            }
            Expr::ObjectLit(entries) => {
                let mut map = BTreeMap::new();
                for (k, v) in entries {
                    map.insert(k.clone(), self.eval_expr(v, scopes, dom, host)?);
                }
                Ok(JsValue::Object(map))
            }
            Expr::FunctionExpr { params, body } => Ok(JsValue::Function {
                params: params.clone(),
                body: body.clone(),
            }),
            Expr::UnaryNot(inner) => {
                let v = self.eval_expr(inner, scopes, dom, host)?;
                Ok(JsValue::Bool(!v.is_truthy()))
            }
            Expr::UnaryNeg(inner) => {
                let v = self.eval_expr(inner, scopes, dom, host)?;
                match v {
                    JsValue::Number(n) => Ok(JsValue::Number(-n)),
                    _ => Ok(JsValue::Number(f64::NAN)),
                }
            }
            Expr::Binary { op, left, right } => {
                if *op == BinOp::And {
                    let l = self.eval_expr(left, scopes, dom, host)?;
                    if !l.is_truthy() {
                        return Ok(l);
                    }
                    return self.eval_expr(right, scopes, dom, host);
                }
                if *op == BinOp::Or {
                    let l = self.eval_expr(left, scopes, dom, host)?;
                    if l.is_truthy() {
                        return Ok(l);
                    }
                    return self.eval_expr(right, scopes, dom, host);
                }
                let l = self.eval_expr(left, scopes, dom, host)?;
                let r = self.eval_expr(right, scopes, dom, host)?;
                Ok(eval_binary_op(*op, l, r))
            }
            Expr::Member { object, property } => {
                let obj = self.eval_expr(object, scopes, dom, host)?;
                match obj {
                    JsValue::DomElement(node_id) => match property.as_str() {
                        "textContent" | "innerText" => Ok(JsValue::Str(dom.text_content(node_id))),
                        "value" => Ok(JsValue::Str(
                            dom.get_attribute(node_id, "value").unwrap_or("").to_owned(),
                        )),
                        "id" => Ok(JsValue::Str(
                            dom.get_attribute(node_id, "id").unwrap_or("").to_owned(),
                        )),
                        "className" => Ok(JsValue::Str(
                            dom.get_attribute(node_id, "class").unwrap_or("").to_owned(),
                        )),
                        "classList" => Ok(JsValue::DomClassList(node_id)),
                        _ => Ok(dom
                            .get_attribute(node_id, property)
                            .map(|s| JsValue::Str(s.to_owned()))
                            .unwrap_or(JsValue::Undefined)),
                    },
                    JsValue::Array(arr) if property == "length" => {
                        Ok(JsValue::Number(arr.len() as f64))
                    }
                    JsValue::Str(s) if property == "length" => {
                        Ok(JsValue::Number(s.chars().count() as f64))
                    }
                    JsValue::Object(map) => {
                        Ok(map.get(property).cloned().unwrap_or(JsValue::Undefined))
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Index { object, index } => {
                let obj = self.eval_expr(object, scopes, dom, host)?;
                let idx = self.eval_expr(index, scopes, dom, host)?;
                match (obj, idx) {
                    (JsValue::Array(arr), JsValue::Number(n)) if n >= 0.0 => {
                        Ok(arr.get(n as usize).cloned().unwrap_or(JsValue::Undefined))
                    }
                    (JsValue::Object(map), JsValue::Str(k)) => {
                        Ok(map.get(&k).cloned().unwrap_or(JsValue::Undefined))
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Call { callee, args } => {
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    evaluated_args.push(self.eval_expr(a, scopes, dom, host)?);
                }

                // Host/DOM method call `obj.method(args...)`
                if let Expr::Member { object, property } = callee.as_ref() {
                    if let Expr::Var(ns) = object.as_ref() {
                        match (ns.as_str(), property.as_str()) {
                            ("document", "getElementById") => {
                                let id = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                return Ok(dom
                                    .get_element_by_id(&id)
                                    .map_or(JsValue::Null, JsValue::DomElement));
                            }
                            ("document", "querySelector") => {
                                let sel_str = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                let sheet = parse_stylesheet(
                                    &format!("{sel_str} {{ color: #000; }}"),
                                    self.budget,
                                )?;
                                if let Some(rule) = sheet.rules.first() {
                                    if let Some(sel) = rule.selectors.first() {
                                        let found = dom.find_in_subtree(dom.root(), &|node| {
                                            selector_matches(dom, node.id, sel)
                                        });
                                        return Ok(found.map_or(JsValue::Null, JsValue::DomElement));
                                    }
                                }
                                return Ok(JsValue::Null);
                            }
                            ("freemvvm", "getProperty") => {
                                let alias = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                return host.get_property(&alias);
                            }
                            ("freemvvm", "setProperty") => {
                                let alias = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                let val =
                                    evaluated_args.get(1).cloned().unwrap_or(JsValue::Undefined);
                                host.set_property(&alias, val)?;
                                return Ok(JsValue::Undefined);
                            }
                            ("freemvvm", "invokeCommand") => {
                                let alias = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                let executed = host.invoke_command(&alias)?;
                                return Ok(JsValue::Bool(executed));
                            }
                            _ => {}
                        }
                    }

                    let recv = self.eval_expr(object, scopes, dom, host)?;
                    match recv {
                        JsValue::DomElement(node_id) => match property.as_str() {
                            "getAttribute" => {
                                let name = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                return Ok(dom
                                    .get_attribute(node_id, &name)
                                    .map(|v| JsValue::Str(v.to_owned()))
                                    .unwrap_or(JsValue::Null));
                            }
                            "setAttribute" => {
                                let name = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                let val = evaluated_args
                                    .get(1)
                                    .map_or_else(String::new, JsValue::to_js_string);
                                dom.set_attribute(node_id, name, val)?;
                                return Ok(JsValue::Undefined);
                            }
                            "addClass" => {
                                let cls = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                dom.add_class(node_id, &cls)?;
                                return Ok(JsValue::Undefined);
                            }
                            "removeClass" => {
                                let cls = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                dom.remove_class(node_id, &cls)?;
                                return Ok(JsValue::Undefined);
                            }
                            "addEventListener" => {
                                let ev_type = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                if let Some(cb) = evaluated_args.get(1) {
                                    self.listeners.push(DomEventListener {
                                        node_id,
                                        event_type: ev_type,
                                        callback: cb.clone(),
                                    });
                                }
                                return Ok(JsValue::Undefined);
                            }
                            _ => {}
                        },
                        JsValue::DomClassList(node_id) => match property.as_str() {
                            "add" => {
                                let cls = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                dom.add_class(node_id, &cls)?;
                                return Ok(JsValue::Undefined);
                            }
                            "remove" => {
                                let cls = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                dom.remove_class(node_id, &cls)?;
                                return Ok(JsValue::Undefined);
                            }
                            "contains" => {
                                let cls = evaluated_args
                                    .first()
                                    .map_or_else(String::new, JsValue::to_js_string);
                                let has = dom
                                    .get_attribute(node_id, "class")
                                    .unwrap_or("")
                                    .split_ascii_whitespace()
                                    .any(|c| c == cls);
                                return Ok(JsValue::Bool(has));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                let fn_val = self.eval_expr(callee, scopes, dom, host)?;
                match fn_val {
                    JsValue::Function { params, body } => {
                        self.call_user_function(&params, &body, &evaluated_args, dom, host)
                    }
                    _ => Err(WebViewError::ScriptRuntimeError(
                        "attempted to call a non-function value".to_owned(),
                    )),
                }
            }
        }
    }

    fn call_user_function(
        &mut self,
        params: &[String],
        body: &[Stmt],
        args: &[JsValue],
        dom: &mut DomArena,
        host: &mut dyn ScriptHostBridge,
    ) -> Result<JsValue, WebViewError> {
        if self.call_depth >= self.budget.max_call_depth {
            return Err(WebViewError::ScriptStackOverflow {
                max_depth: self.budget.max_call_depth,
            });
        }
        self.call_depth += 1;
        let mut frame = BTreeMap::new();
        for (idx, param) in params.iter().enumerate() {
            frame.insert(
                param.clone(),
                args.get(idx).cloned().unwrap_or(JsValue::Undefined),
            );
        }
        let mut fn_scopes = vec![frame];
        let result = self.exec_block(body, &mut fn_scopes, dom, host);
        self.call_depth -= 1;
        let (ret, _) = result?;
        Ok(ret.unwrap_or(JsValue::Undefined))
    }
}

fn eval_binary_op(op: BinOp, left: JsValue, right: JsValue) -> JsValue {
    match op {
        BinOp::Add => match (&left, &right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a + b),
            _ => JsValue::Str(format!("{}{}", left.to_js_string(), right.to_js_string())),
        },
        BinOp::Sub => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a - b),
            _ => JsValue::Number(f64::NAN),
        },
        BinOp::Mul => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a * b),
            _ => JsValue::Number(f64::NAN),
        },
        BinOp::Div => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a / b),
            _ => JsValue::Number(f64::NAN),
        },
        BinOp::Rem => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a % b),
            _ => JsValue::Number(f64::NAN),
        },
        BinOp::Eq => JsValue::Bool(left == right),
        BinOp::Ne => JsValue::Bool(left != right),
        BinOp::Lt => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Bool(a < b),
            (JsValue::Str(a), JsValue::Str(b)) => JsValue::Bool(a < b),
            _ => JsValue::Bool(false),
        },
        BinOp::Le => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Bool(a <= b),
            (JsValue::Str(a), JsValue::Str(b)) => JsValue::Bool(a <= b),
            _ => JsValue::Bool(false),
        },
        BinOp::Gt => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Bool(a > b),
            (JsValue::Str(a), JsValue::Str(b)) => JsValue::Bool(a > b),
            _ => JsValue::Bool(false),
        },
        BinOp::Ge => match (left, right) {
            (JsValue::Number(a), JsValue::Number(b)) => JsValue::Bool(a >= b),
            (JsValue::Str(a), JsValue::Str(b)) => JsValue::Bool(a >= b),
            _ => JsValue::Bool(false),
        },
        BinOp::And | BinOp::Or => unreachable!(),
    }
}

fn tokenize(src: &str) -> Result<Vec<Token>, WebViewError> {
    let mut tokens = Vec::new();
    let bytes = src.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        let b = bytes[idx];
        if b.is_ascii_whitespace() {
            idx += 1;
            continue;
        }
        if src[idx..].starts_with("//") {
            while idx < bytes.len() && bytes[idx] != b'\n' {
                idx += 1;
            }
            continue;
        }
        if src[idx..].starts_with("/*") {
            if let Some(end) = src[idx + 2..].find("*/") {
                idx += 2 + end + 2;
                continue;
            } else {
                return Err(WebViewError::SyntaxError(
                    "unterminated block comment".to_owned(),
                ));
            }
        }

        for (pat, tok) in [
            ("===", Token::EqStrict),
            ("!==", Token::NeStrict),
            ("==", Token::EqLoose),
            ("!=", Token::NeLoose),
            ("<=", Token::Le),
            (">=", Token::Ge),
            ("&&", Token::And),
            ("||", Token::Or),
            ("+=", Token::PlusAssign),
            ("-=", Token::MinusAssign),
        ] {
            if src[idx..].starts_with(pat) {
                tokens.push(tok);
                idx += pat.len();
                break;
            }
        }
        if idx < bytes.len() && !bytes[idx].is_ascii_whitespace() && tokens.last().is_some() {
            // Check if we just matched a multi-char operator above
        }
        if idx >= bytes.len() {
            break;
        }

        // Re-check multi-char cleanly:
        let rest = &src[idx..];
        if rest.starts_with("===")
            || rest.starts_with("!==")
            || rest.starts_with("==")
            || rest.starts_with("!=")
            || rest.starts_with("<=")
            || rest.starts_with(">=")
            || rest.starts_with("&&")
            || rest.starts_with("||")
            || rest.starts_with("+=")
            || rest.starts_with("-=")
        {
            continue;
        }

        match bytes[idx] {
            b'(' => {
                tokens.push(Token::LParen);
                idx += 1;
            }
            b')' => {
                tokens.push(Token::RParen);
                idx += 1;
            }
            b'{' => {
                tokens.push(Token::LBrace);
                idx += 1;
            }
            b'}' => {
                tokens.push(Token::RBrace);
                idx += 1;
            }
            b'[' => {
                tokens.push(Token::LBracket);
                idx += 1;
            }
            b']' => {
                tokens.push(Token::RBracket);
                idx += 1;
            }
            b';' => {
                tokens.push(Token::Semi);
                idx += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                idx += 1;
            }
            b'.' => {
                tokens.push(Token::Dot);
                idx += 1;
            }
            b':' => {
                tokens.push(Token::Colon);
                idx += 1;
            }
            b'=' => {
                tokens.push(Token::Assign);
                idx += 1;
            }
            b'+' => {
                tokens.push(Token::Plus);
                idx += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                idx += 1;
            }
            b'*' => {
                tokens.push(Token::Star);
                idx += 1;
            }
            b'/' => {
                tokens.push(Token::Slash);
                idx += 1;
            }
            b'%' => {
                tokens.push(Token::Percent);
                idx += 1;
            }
            b'<' => {
                tokens.push(Token::Lt);
                idx += 1;
            }
            b'>' => {
                tokens.push(Token::Gt);
                idx += 1;
            }
            b'!' => {
                tokens.push(Token::Not);
                idx += 1;
            }
            b'"' | b'\'' => {
                let quote = bytes[idx];
                idx += 1;
                let mut s = String::new();
                while idx < bytes.len() && bytes[idx] != quote {
                    if bytes[idx] == b'\\' && idx + 1 < bytes.len() {
                        idx += 1;
                        let esc = match bytes[idx] {
                            b'n' => '\n',
                            b't' => '\t',
                            b'r' => '\r',
                            b'"' => '"',
                            b'\'' => '\'',
                            b'\\' => '\\',
                            other => other as char,
                        };
                        s.push(esc);
                        idx += 1;
                    } else {
                        let ch = src[idx..].chars().next().unwrap();
                        s.push(ch);
                        idx += ch.len_utf8();
                    }
                }
                if idx >= bytes.len() {
                    return Err(WebViewError::SyntaxError(
                        "unterminated string literal".to_owned(),
                    ));
                }
                idx += 1;
                tokens.push(Token::Str(s));
            }
            b if b.is_ascii_digit() => {
                let start = idx;
                while idx < bytes.len() && (bytes[idx].is_ascii_digit() || bytes[idx] == b'.') {
                    idx += 1;
                }
                let num = src[start..idx]
                    .parse::<f64>()
                    .map_err(|_| WebViewError::SyntaxError("invalid number literal".to_owned()))?;
                tokens.push(Token::Number(num));
            }
            b if b.is_ascii_alphabetic() || b == b'_' || b == b'$' => {
                let start = idx;
                while idx < bytes.len()
                    && (bytes[idx].is_ascii_alphanumeric()
                        || bytes[idx] == b'_'
                        || bytes[idx] == b'$')
                {
                    idx += 1;
                }
                let ident = &src[start..idx];
                // Disallow `eval` and `Function` constructor at the syntax level (`SD-WV-04`)
                if ident == "eval" || ident == "Function" {
                    return Err(WebViewError::SyntaxError(format!(
                        "dynamic code generation '{ident}' is forbidden in WebView sandbox"
                    )));
                }
                let tok = match ident {
                    "let" => Token::Let,
                    "const" => Token::Const,
                    "var" => Token::Var,
                    "function" => Token::Function,
                    "return" => Token::Return,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "true" => Token::True,
                    "false" => Token::False,
                    "null" => Token::Null,
                    "undefined" => Token::Undefined,
                    _ => Token::Ident(ident.to_owned()),
                };
                tokens.push(tok);
            }
            other => {
                return Err(WebViewError::SyntaxError(format!(
                    "unexpected character '{}'",
                    other as char
                )));
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

fn parse_program(src: &str) -> Result<Vec<Stmt>, WebViewError> {
    let tokens = tokenize(src)?;
    let mut parser = Parser { tokens, pos: 0 };
    let mut stmts = Vec::new();
    while parser.peek().is_some() {
        stmts.push(parser.parse_stmt()?);
    }
    Ok(stmts)
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), WebViewError> {
        match self.next() {
            Some(ref t) if t == expected => Ok(()),
            other => Err(WebViewError::SyntaxError(format!(
                "expected {expected:?}, got {other:?}"
            ))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, WebViewError> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(name),
            other => Err(WebViewError::SyntaxError(format!(
                "expected identifier, got {other:?}"
            ))),
        }
    }

    fn eat_semi(&mut self) {
        if self.peek() == Some(&Token::Semi) {
            self.pos += 1;
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, WebViewError> {
        match self.peek() {
            Some(Token::Let | Token::Const | Token::Var) => {
                self.next();
                let name = self.expect_ident()?;
                let init = if self.peek() == Some(&Token::Assign) {
                    self.next();
                    self.parse_expr()?
                } else {
                    Expr::Literal(JsValue::Undefined)
                };
                self.eat_semi();
                Ok(Stmt::VarDecl { name, init })
            }
            Some(Token::Function) => {
                self.next();
                let name = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                while self.peek() != Some(&Token::RParen) {
                    params.push(self.expect_ident()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.next();
                    }
                }
                self.expect(&Token::RParen)?;
                let body = self.parse_block()?;
                Ok(Stmt::FuncDecl { name, params, body })
            }
            Some(Token::If) => {
                self.next();
                self.expect(&Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                let then_branch = self.parse_block_or_single()?;
                let else_branch = if self.peek() == Some(&Token::Else) {
                    self.next();
                    self.parse_block_or_single()?
                } else {
                    Vec::new()
                };
                Ok(Stmt::If {
                    cond,
                    then_branch,
                    else_branch,
                })
            }
            Some(Token::While) => {
                self.next();
                self.expect(&Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                let body = self.parse_block_or_single()?;
                Ok(Stmt::While { cond, body })
            }
            Some(Token::For) => {
                self.next();
                self.expect(&Token::LParen)?;
                let init = if self.peek() == Some(&Token::Semi) {
                    self.next();
                    None
                } else {
                    Some(Box::new(self.parse_stmt()?))
                };
                let cond = if self.peek() == Some(&Token::Semi) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.eat_semi();
                let step = if self.peek() == Some(&Token::RParen) {
                    None
                } else {
                    Some(Box::new(self.parse_stmt()?))
                };
                self.expect(&Token::RParen)?;
                let body = self.parse_block_or_single()?;
                Ok(Stmt::For {
                    init,
                    cond,
                    step,
                    body,
                })
            }
            Some(Token::Return) => {
                self.next();
                if matches!(self.peek(), Some(Token::Semi | Token::RBrace) | None) {
                    self.eat_semi();
                    Ok(Stmt::Return(None))
                } else {
                    let e = self.parse_expr()?;
                    self.eat_semi();
                    Ok(Stmt::Return(Some(e)))
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                match self.peek() {
                    Some(Token::Assign | Token::PlusAssign | Token::MinusAssign) => {
                        let op_tok = self.next().unwrap();
                        let rhs = self.parse_expr()?;
                        self.eat_semi();
                        let combined_rhs = match op_tok {
                            Token::Assign => rhs,
                            Token::PlusAssign => Expr::Binary {
                                op: BinOp::Add,
                                left: Box::new(expr.clone()),
                                right: Box::new(rhs),
                            },
                            Token::MinusAssign => Expr::Binary {
                                op: BinOp::Sub,
                                left: Box::new(expr.clone()),
                                right: Box::new(rhs),
                            },
                            _ => unreachable!(),
                        };
                        match expr {
                            Expr::Var(name) => Ok(Stmt::Assign {
                                name,
                                expr: combined_rhs,
                            }),
                            Expr::Member { object, property } => Ok(Stmt::MemberAssign {
                                object: *object,
                                property,
                                expr: combined_rhs,
                            }),
                            _ => Err(WebViewError::SyntaxError(
                                "invalid assignment target".to_owned(),
                            )),
                        }
                    }
                    _ => {
                        self.eat_semi();
                        Ok(Stmt::ExprStmt(expr))
                    }
                }
            }
        }
    }

    fn parse_block_or_single(&mut self) -> Result<Vec<Stmt>, WebViewError> {
        if self.peek() == Some(&Token::LBrace) {
            self.parse_block()
        } else {
            Ok(vec![self.parse_stmt()?])
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, WebViewError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek().is_some() && self.peek() != Some(&Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_expr(&mut self) -> Result<Expr, WebViewError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.next();
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_equality()?;
        while self.peek() == Some(&Token::And) {
            self.next();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_comparison()?;
        while let Some(Token::EqStrict | Token::EqLoose | Token::NeStrict | Token::NeLoose) =
            self.peek()
        {
            let op = match self.next().unwrap() {
                Token::EqStrict | Token::EqLoose => BinOp::Eq,
                _ => BinOp::Ne,
            };
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_additive()?;
        while let Some(Token::Lt | Token::Le | Token::Gt | Token::Ge) = self.peek() {
            let op = match self.next().unwrap() {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_multiplicative()?;
        while let Some(Token::Plus | Token::Minus) = self.peek() {
            let op = match self.next().unwrap() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, WebViewError> {
        let mut left = self.parse_unary()?;
        while let Some(Token::Star | Token::Slash | Token::Percent) = self.peek() {
            let op = match self.next().unwrap() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Rem,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, WebViewError> {
        match self.peek() {
            Some(Token::Not) => {
                self.next();
                Ok(Expr::UnaryNot(Box::new(self.parse_unary()?)))
            }
            Some(Token::Minus) => {
                self.next();
                Ok(Expr::UnaryNeg(Box::new(self.parse_unary()?)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, WebViewError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.next();
                    let prop = self.expect_ident()?;
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: prop,
                    };
                }
                Some(Token::LBracket) => {
                    self.next();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(Token::LParen) => {
                    self.next();
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.next();
                        }
                    }
                    self.expect(&Token::RParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, WebViewError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expr::Literal(JsValue::Number(n))),
            Some(Token::Str(s)) => Ok(Expr::Literal(JsValue::Str(s))),
            Some(Token::True) => Ok(Expr::Literal(JsValue::Bool(true))),
            Some(Token::False) => Ok(Expr::Literal(JsValue::Bool(false))),
            Some(Token::Null) => Ok(Expr::Literal(JsValue::Null)),
            Some(Token::Undefined) => Ok(Expr::Literal(JsValue::Undefined)),
            Some(Token::Ident(name)) => Ok(Expr::Var(name)),
            Some(Token::LParen) => {
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            Some(Token::LBracket) => {
                let mut items = Vec::new();
                while self.peek() != Some(&Token::RBracket) {
                    items.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.next();
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::ArrayLit(items))
            }
            Some(Token::LBrace) => {
                let mut entries = Vec::new();
                while self.peek() != Some(&Token::RBrace) {
                    let key = match self.next() {
                        Some(Token::Ident(k) | Token::Str(k)) => k,
                        other => {
                            return Err(WebViewError::SyntaxError(format!(
                                "expected object key, got {other:?}"
                            )));
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let val = self.parse_expr()?;
                    entries.push((key, val));
                    if self.peek() == Some(&Token::Comma) {
                        self.next();
                    }
                }
                self.expect(&Token::RBrace)?;
                Ok(Expr::ObjectLit(entries))
            }
            Some(Token::Function) => {
                if let Some(Token::Ident(_)) = self.peek() {
                    self.next();
                }
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                while self.peek() != Some(&Token::RParen) {
                    params.push(self.expect_ident()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.next();
                    }
                }
                self.expect(&Token::RParen)?;
                let body = self.parse_block()?;
                Ok(Expr::FunctionExpr { params, body })
            }
            other => Err(WebViewError::SyntaxError(format!(
                "unexpected token {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::parse_html;

    #[test]
    fn executes_script_dom_manipulation_events_and_mvvm_bridge() {
        let html = r#"
            <div>
                <span id="counter">0</span>
                <button id="btn">Increment</button>
            </div>
        "#;
        let mut doc = parse_html(html, ResourceBudget::default()).unwrap();
        let mut host = InMemoryHostBridge::new();
        host.expose_property("count", JsValue::Number(10.0), true);
        host.expose_command("saveCmd", true);

        let mut vm = ScriptVm::new(ResourceBudget::default());
        let script = r##"
            let el = document.getElementById("counter");
            let initial = freemvvm.getProperty("count");
            el.textContent = "Count: " + initial;
            el.classList.add("active");

            let btn = document.querySelector("#btn");
            btn.addEventListener("click", function() {
                let next = freemvvm.getProperty("count") + 5;
                freemvvm.setProperty("count", next);
                el.textContent = "Count: " + next;
                freemvvm.invokeCommand("saveCmd");
            });
        "##;

        vm.eval_script(script, &mut doc.dom, &mut host).unwrap();
        let counter_id = doc.dom.get_element_by_id("counter").unwrap();
        assert_eq!(doc.dom.text_content(counter_id), "Count: 10");
        assert_eq!(doc.dom.get_attribute(counter_id, "class"), Some("active"));

        let btn_id = doc.dom.get_element_by_id("btn").unwrap();
        vm.dispatch_dom_event(btn_id, "click", &mut doc.dom, &mut host)
            .unwrap();
        assert_eq!(doc.dom.text_content(counter_id), "Count: 15");
        assert_eq!(host.get_property("count"), Ok(JsValue::Number(15.0)));
        assert_eq!(host.invoked_commands(), &["saveCmd"]);
    }

    #[test]
    fn aborts_infinite_loop_and_rolls_back_uncommitted_dom_mutations() {
        let html = r#"<p id="msg">Original</p>"#;
        let mut doc = parse_html(html, ResourceBudget::default()).unwrap();
        let mut host = InMemoryHostBridge::new();
        let budget = ResourceBudget {
            max_script_steps: 250,
            max_call_depth: 8,
            ..ResourceBudget::default()
        };
        let mut vm = ScriptVm::new(budget);

        let bad_script = r#"
            let p = document.getElementById("msg");
            p.textContent = "Mutated before infinite loop";
            while (true) {}
        "#;
        let res = vm.eval_script(bad_script, &mut doc.dom, &mut host);
        assert_eq!(
            res,
            Err(WebViewError::ScriptStepLimitExceeded { max_steps: 250 })
        );
        let msg_id = doc.dom.get_element_by_id("msg").unwrap();
        // DOM must be rolled back to "Original"
        assert_eq!(doc.dom.text_content(msg_id), "Original");

        // Deep recursion is also caught
        let rec_script = "function inf() { return inf(); } inf();";
        assert_eq!(
            vm.eval_script(rec_script, &mut doc.dom, &mut host),
            Err(WebViewError::ScriptStackOverflow { max_depth: 8 })
        );

        // `eval` is rejected at parse time
        assert!(matches!(
            vm.eval_script("eval('1 + 1')", &mut doc.dom, &mut host),
            Err(WebViewError::SyntaxError(_))
        ));
    }
}
