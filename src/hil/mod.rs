mod bui_builder;

use std::sync::atomic::{AtomicU32, Ordering};

use lexpr::{sexp, Cons, Number, Value};

use crate::bui::BuiMessage;
use crate::hil::bui_builder::{ItemCollectVisitor, ItemCountVisitor};
use crate::s_expr::node_get_field;

pub type Hil = Value; // high level intermediate language
pub type Node = Cons;

#[derive(Clone)]
pub struct UnitNum(u16);

const LOCAL_UNIT: u16 = 0;

impl UnitNum {
    fn new(x: u16) -> Self {
        UnitNum { 0: x }
    }
}

#[derive(Clone, Copy)]
pub struct DefLocalId(u32);

impl DefLocalId {
    pub fn new() -> Self {
        DefLocalId { 0: 0 }
    }

    pub fn next(&self) -> DefLocalId {
        DefLocalId { 0: self.0 + 1 }
    }

    pub fn to_def_id(&self) -> DefId {
        DefId {
            unit: UnitNum::new(LOCAL_UNIT),
            offset: *self,
        }
    }
}

#[derive(Clone)]
pub struct DefId {
    unit: UnitNum,
    offset: DefLocalId,
}

impl DefId {
    #[rustfmt::skip]
    pub fn to_s_expr(&self) -> Value {
        let unit = self.unit.0;
        let offset = self.offset.0;
        sexp!((DefId ,unit ,offset))
    }
    pub fn from_s_expr(value: &Value) -> Self {
        let node = value.as_cons().unwrap();
        assert_eq!(node.car().as_symbol().unwrap(), "DefId");
        let unit = node_get_field(&node, 0).as_u64().unwrap();
        let offset = node_get_field(&node, 1).as_u64().unwrap();
        DefId {
            unit: UnitNum::new(u16::try_from(unit).unwrap()),
            offset: DefLocalId(u32::try_from(offset).unwrap()),
        }
    }
}

pub fn item_ref_opaque(def_id: DefId) -> (u16, u32) {
    (def_id.unit.0, def_id.offset.0)
}

static HIL_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy)]
pub struct HilId {
    private: u32,
}

impl HilId {
    pub fn new() -> Self {
        HilId {
            private: HIL_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn s_expr(&self) -> Value {
        Value::Number(Number::from(self.private))
    }
}

pub trait HilVisitor {
    fn visit(&mut self, hil: &Hil) {
        walk_hil(self, hil)
    }
    fn visit_item(&mut self, _: &Node) {}
    fn visit_pre_namespace(&mut self, _: &Node) {}
    fn visit_post_namespace(&mut self, _: &Node) {}
}

fn walk_hil<T: ?Sized + HilVisitor>(v: &mut T, hil: &Hil) {
    match hil {
        Value::Cons(c) => {
            if let Some(sym) = c.car().as_symbol() {
                if sym == "Item" {
                    v.visit_item(c);
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

pub fn to_bui(hil: &Hil) -> BuiMessage {
    let mut counter = ItemCountVisitor::new();
    counter.visit(hil);
    let mut collector = ItemCollectVisitor::new(counter.get());
    collector.visit(hil);
    collector.collect()
}
