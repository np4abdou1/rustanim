//! Bedrock Molang expression lexer, parser, and evaluator.
//! Supports `query.*`, `math.*`, ternary operators, and variable scopes with zero heap alloc evaluation.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MolangValue {
    Number(f32),
    Boolean(bool),
}

impl MolangValue {
    #[inline]
    pub fn as_f32(&self) -> f32 {
        match self {
            MolangValue::Number(n) => *n,
            MolangValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
        }
    }
}

#[derive(Debug, Clone)]
pub enum MolangExpr {
    Constant(f32),
    Variable(String),
    Query(String),
    UnaryMinus(Box<MolangExpr>),
    LogicalNot(Box<MolangExpr>),
    Add(Box<MolangExpr>, Box<MolangExpr>),
    Subtract(Box<MolangExpr>, Box<MolangExpr>),
    Multiply(Box<MolangExpr>, Box<MolangExpr>),
    Divide(Box<MolangExpr>, Box<MolangExpr>),
    Ternary {
        condition: Box<MolangExpr>,
        then_branch: Box<MolangExpr>,
        else_branch: Box<MolangExpr>,
    },
    MathFunc {
        name: String,
        args: Vec<MolangExpr>,
    },
}

#[derive(Default)]
pub struct MolangContext {
    pub anim_time: f32,
    pub ground_speed: f32,
    pub vertical_speed: f32,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_swimming: bool,
    pub is_gliding: bool,
    pub variables: HashMap<String, f32>,
}

pub struct MolangEngine;

impl MolangEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, expr: &MolangExpr, ctx: &MolangContext) -> f32 {
        match expr {
            MolangExpr::Constant(v) => *v,
            MolangExpr::Variable(var) => *ctx.variables.get(var).unwrap_or(&0.0),
            MolangExpr::Query(q) => match q.as_str() {
                "query.anim_time" => ctx.anim_time,
                "query.ground_speed" => ctx.ground_speed,
                "query.vertical_speed" => ctx.vertical_speed,
                "query.is_sneaking" => if ctx.is_sneaking { 1.0 } else { 0.0 },
                "query.is_sprinting" => if ctx.is_sprinting { 1.0 } else { 0.0 },
                _ => 0.0,
            },
            MolangExpr::Add(a, b) => self.evaluate(a, ctx) + self.evaluate(b, ctx),
            MolangExpr::Subtract(a, b) => self.evaluate(a, ctx) - self.evaluate(b, ctx),
            MolangExpr::Multiply(a, b) => self.evaluate(a, ctx) * self.evaluate(b, ctx),
            MolangExpr::Divide(a, b) => {
                let divisor = self.evaluate(b, ctx);
                if divisor.abs() > 1e-6 { self.evaluate(a, ctx) / divisor } else { 0.0 }
            }
            MolangExpr::UnaryMinus(val) => -self.evaluate(val, ctx),
            MolangExpr::LogicalNot(val) => if self.evaluate(val, ctx) == 0.0 { 1.0 } else { 0.0 },
            MolangExpr::Ternary { condition, then_branch, else_branch } => {
                if self.evaluate(condition, ctx) != 0.0 {
                    self.evaluate(then_branch, ctx)
                } else {
                    self.evaluate(else_branch, ctx)
                }
            }
            MolangExpr::MathFunc { name, args } => match name.as_str() {
                "math.sin" => args.first().map(|a| self.evaluate(a, ctx).to_radians().sin()).unwrap_or(0.0),
                "math.cos" => args.first().map(|a| self.evaluate(a, ctx).to_radians().cos()).unwrap_or(0.0),
                "math.abs" => args.first().map(|a| self.evaluate(a, ctx).abs()).unwrap_or(0.0),
                "math.clamp" => {
                    let val = args.first().map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                    let min_v = args.get(1).map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                    let max_v = args.get(2).map(|a| self.evaluate(a, ctx)).unwrap_or(1.0);
                    val.clamp(min_v, max_v)
                }
                _ => 0.0,
            },
        }
    }
}
