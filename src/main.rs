//! expr-typecheck — Simple type checker for arithmetic/boolean/string expressions.

use std::fmt;

/// Types in the expression language.
#[derive(Debug, Clone, PartialEq)]
enum Type {
    Number,
    Bool,
    String,
    Unit,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Unit => write!(f, "()"),
        }
    }
}

/// Binary operators with type signatures.
#[derive(Debug, Clone, Copy)]
enum BinOp { Add, Sub, Mul, Div, Eq, Ne, Lt, Gt, And, Or, Concat }

/// Unary operators.
#[derive(Debug, Clone, Copy)]
enum UnOp { Neg, Not, Len }

/// Typed expression AST.
#[derive(Debug, Clone)]
enum Expr {
    LitNum(f64),
    LitBool(bool),
    LitStr(String),
    Var(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    If { cond: Box<Expr>, then_br: Box<Expr>, else_br: Option<Box<Expr>> },
    Let { name: String, value: Box<Expr>, body: Box<Expr> },
}

/// Type-check error.
#[derive(Debug)]
enum TypeError {
    Mismatch { expected: Type, got: Type, context: String },
    UndefinedVar(String),
    NonBoolCond(Type),
    BranchMismatch { then_type: Type, else_type: Type },
    UnsupportedOp { op: BinOp, left: Type, right: Type },
    UnsupportedUnOp { op: UnOp, operand: Type },
}

use std::collections::HashMap;

/// Type-checking context with variable bindings.
struct TypeChecker {
    env: HashMap<String, Type>,
}

impl TypeChecker {
    fn new() -> Self { Self { env: HashMap::new() } }

    fn check(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::LitNum(_) => Ok(Type::Number),
            Expr::LitBool(_) => Ok(Type::Bool),
            Expr::LitStr(_) => Ok(Type::String),
            Expr::Var(name) => self.env.get(name).cloned().ok_or_else(|| TypeError::UndefinedVar(name.clone())),
            Expr::Unary { op, operand } => {
                let t = self.check(operand)?;
                match op {
                    UnOp::Neg if t == Type::Number => Ok(Type::Number),
                    UnOp::Not if t == Type::Bool => Ok(Type::Bool),
                    _ => Err(TypeError::UnsupportedUnOp { op: *op, operand: t }),
                }
            }
            Expr::Binary { op, left, right } => {
                let lt = self.check(left)?;
                let rt = self.check(right)?;
                self.check_binop(*op, lt, rt)
            }
            Expr::If { cond, then_br, else_br } => {
                let ct = self.check(cond)?;
                if ct != Type::Bool { return Err(TypeError::NonBoolCond(ct)); }
                let tt = self.check(then_br)?;
                if let Some(eb) = else_br {
                    let et = self.check(eb)?;
                    if tt != et { return Err(TypeError::BranchMismatch { then_type: tt, else_type: et }); }
                    Ok(tt)
                } else { Ok(Type::Unit) }
            }
            Expr::Let { name, value, body } => {
                let vt = self.check(value)?;
                self.env.insert(name.clone(), vt);
                let bt = self.check(body)?;
                self.env.remove(name);
                Ok(bt)
            }
        }
    }

    fn check_binop(&self, op: BinOp, lt: Type, rt: Type) -> Result<Type, TypeError> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if lt == Type::Number && rt == Type::Number { Ok(Type::Number) }
                else { Err(TypeError::UnsupportedOp { op, left: lt, right: rt }) }
            }
            BinOp::Eq | BinOp::Ne => {
                if lt == rt { Ok(Type::Bool) } else { Err(TypeError::UnsupportedOp { op, left: lt, right: rt }) }
            }
            BinOp::Lt | BinOp::Gt => {
                if lt == Type::Number && rt == Type::Number { Ok(Type::Bool) }
                else { Err(TypeError::UnsupportedOp { op, left: lt, right: rt }) }
            }
            BinOp::And | BinOp::Or => {
                if lt == Type::Bool && rt == Type::Bool { Ok(Type::Bool) }
                else { Err(TypeError::UnsupportedOp { op, left: lt, right: rt }) }
            }
            BinOp::Concat => {
                if lt == Type::String && rt == Type::String { Ok(Type::String) }
                else { Err(TypeError::UnsupportedOp { op, left: lt, right: rt }) }
            }
        }
    }
}

fn main() {
    let mut tc = TypeChecker::new();
    let tests: Vec<(Expr, &str)> = vec![
        (Expr::Binary { op: BinOp::Add, left: Box::new(Expr::LitNum(1.0)), right: Box::new(Expr::LitNum(2.0)) }, "1 + 2"),
        (Expr::Binary { op: BinOp::Lt, left: Box::new(Expr::LitNum(1.0)), right: Box::new(Expr::LitNum(2.0)) }, "1 < 2"),
        (Expr::Binary { op: BinOp::Add, left: Box::new(Expr::LitNum(1.0)), right: Box::new(Expr::LitBool(true)) }, "1 + true"),
        (Expr::Binary { op: BinOp::Concat, left: Box::new(Expr::LitStr("hello".into())), right: Box::new(Expr::LitStr(" world".into())) }, "\"hello\" ++ \" world\""),
        (Expr::Unary { op: UnOp::Not, operand: Box::new(Expr::Binary { op: BinOp::Eq, left: Box::new(Expr::LitNum(1.0)), right: Box::new(Expr::LitNum(2.0)) }) }, "!(1 == 2)"),
    ];

    println!("expr-typecheck — type checker");
    println!("=============================\n");
    for (expr, desc) in &tests {
        match tc.check(expr) {
            Ok(t) => println!("{desc:<30} : {t}"),
            Err(e) => println!("{desc:<30} : ERROR — {e:?}"),
        }
    }
}
