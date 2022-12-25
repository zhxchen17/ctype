pub mod resolve;

use crate::ast::resolve::{lower_ast, LoweringContext, ResolveCollectVisitor, ResolvePathVisitor};
use crate::error::dump_errors;
use crate::hil::Hil;

use lexpr::{Cons, Value};

pub type Ast = Value;
pub type Node = Cons;

pub trait AstVisitor {
    fn visit(&mut self, ast: &Ast) {
        walk_ast(self, ast)
    }
    fn visit_item(&mut self, _: &Node) {}
    fn visit_expr(&mut self, _: &Node) {}
    fn visit_type(&mut self, _: &Node) {}
    fn visit_param(&mut self, _: &Node) {}
    fn visit_pre_namespace(&mut self, _: &Node) {}
    fn visit_post_namespace(&mut self, _: &Node) {}
}

fn walk_ast<T: ?Sized + AstVisitor>(v: &mut T, ast: &Ast) {
    match ast {
        Value::Cons(c) => {
            if let Some(sym) = c.car().as_symbol() {
                if sym == "Item" {
                    v.visit_item(c);
                } else if sym == "Expr" {
                    v.visit_expr(c);
                } else if sym == "Ty" {
                    v.visit_type(c);
                } else if sym == "Param" {
                    v.visit_param(c);
                } else if sym == "Namespace" {
                    v.visit_pre_namespace(c);
                }
            }
            c.list_iter().for_each(|x| v.visit(x));
            if let Some(sym) = c.car().as_symbol() {
                if sym == "Namespace" {
                    v.visit_post_namespace(c);
                }
            }
        }
        Value::Vector(l) => {
            l.iter().for_each(|x| v.visit(x));
        }
        _ => (),
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct NodeId {
    private: *const Node,
}

impl NodeId {
    pub fn new(node: &Node) -> Self {
        NodeId {
            private: node as *const Node,
        }
    }
}

pub fn to_hil(ast: &Ast) -> Hil {
    let mut collector = ResolveCollectVisitor::new();
    collector.visit(ast);
    let mut resolver = ResolvePathVisitor::new(collector.collect());
    resolver.visit(ast);
    dump_errors();
    lower_ast(ast, &mut LoweringContext::new(resolver.resolve()))
}
