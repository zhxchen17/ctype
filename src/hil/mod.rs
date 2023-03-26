mod bui_builder;

use std::sync::atomic::{AtomicU32, Ordering};

use lexpr::{Cons, Number, Value};

use crate::bui::BuiMessage;
use crate::context::GlobalContext;
use crate::def::DefId;
use crate::hil::bui_builder::ToBuiVisitor;
use crate::s_expr::{node_get_attr, node_get_fields};
use crate::t::TypeCheckContext;
use crate::t::context::UnitPath;
use crate::til::Til;

pub type Hil = Value; // high level intermediate language

pub fn hil_get_unit_path(hil: &Hil) -> UnitPath {
    let path = node_get_attr(hil.as_cons().unwrap(), "path");
    UnitPath::new(
        path.as_slice()
            .unwrap()
            .iter()
            .map(|x| x.as_symbol().unwrap().to_string())
            .collect(),
    )
}

pub type Node = Cons;

static HIL_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HilId {
    private: u32,
}

impl HilId {
    pub fn new() -> Self {
        HilId {
            private: HIL_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn to_s_expr(&self) -> Value {
        Value::Number(Number::from(self.private))
    }

    pub fn from_s_expr(value: &Value) -> Self {
        HilId {
            private: u32::try_from(value.as_u64().unwrap()).unwrap()
        }
    }
}

pub trait HilVisitor {
    fn visit(&mut self, hil: &Hil) {
        walk_hil(self, hil)
    }
    fn visit_item(&mut self, _: &Node) {}
    fn visit_pre_namespace(&mut self, _: &Node) {}
    fn visit_post_namespace(&mut self, _: &Node) {}
    fn visit_binding(&mut self, _: &Node) {}
    fn visit_body(&mut self, _: &Node) {}
    fn visit_fn_sig(&mut self, _: &Node) {}
    fn visit_variant(&mut self, _: &Node) {}
    fn visit_field_def(&mut self, _:&Node) {}
    fn visit_pre_block(&mut self, _: &Node) {}
    fn visit_post_block(&mut self, _: &Node) {}
    fn visit_stmt(&mut self, _: &Node) {}
}

pub fn walk_hil_node<T: ?Sized + HilVisitor>(v: &mut T, node: &Node) {
    if let Some(sym) = node.car().as_symbol() {
        if sym == "Item" {
            v.visit_item(node);
        } else if sym == "Namespace" {
            v.visit_pre_namespace(node);
        } else if sym == "Binding" {
            v.visit_binding(node);
        } else if sym == "Body" {
            v.visit_body(node);
        } else if sym == "FnSig" {
            v.visit_fn_sig(node);
        } else if sym == "Variant" {
            v.visit_variant(node);
        } else if sym == "FieldDef" {
            v.visit_field_def(node);
        } else if sym == "Block" {
            v.visit_pre_block(node);
        } else if sym == "Stmt" {
            v.visit_stmt(node);
        }
    }
    node.list_iter().for_each(|x| v.visit(x));
    if let Some(sym) = node.car().as_symbol() {
        if sym == "Namespace" {
            v.visit_post_namespace(node);
        } else if sym == "Block" {
            v.visit_post_block(node);
        }
    }
}

fn walk_hil<T: ?Sized + HilVisitor>(v: &mut T, hil: &Hil) {
    match hil {
        Value::Cons(c) => {
            walk_hil_node(v, c);
        }
        Value::Vector(l) => {
            l.iter().for_each(|x| v.visit(x));
        }
        _ => (),
    }
}

pub fn to_bui<'a>(hil: &Hil, ctx: &'a GlobalContext<'a>) -> BuiMessage {
    assert_eq!(hil.as_cons().unwrap().car().as_symbol().unwrap(), "Unit");
    let fields = node_get_fields(hil.as_cons().unwrap());
    fields.iter().for_each(|x| assert_eq!(x.as_cons().unwrap().car().as_symbol().unwrap(), "Item"));
    let mut collector = ToBuiVisitor::new(fields.len(), ctx);
    collector.visit(hil);
    collector.collect()
}

pub fn to_til<'a>(hil: &Hil, ctx: &'a GlobalContext<'a>, tctx: &TypeCheckContext<'a>) -> Til {
    unimplemented!()
}
