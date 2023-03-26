use std::collections::HashMap;
use crate::context::GlobalContext;
use crate::hil::{Node, HilId};
use crate::s_expr::{node_get_attr, node_get_field};
use crate::t::{TypeRef, TypeKind};

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct UnitPath {
    name: Vec<String>,
}

impl UnitPath {
    pub fn new(name: Vec<String>) -> Self {
        UnitPath { name }
    }
}

pub struct TyCtx<'gcx> {
    global_ctx: &'gcx GlobalContext<'gcx>,
    locals: HashMap<HilId, TypeRef<'gcx>>,
}

impl<'gcx> TyCtx<'gcx> {
    pub fn new(global_ctx: &'gcx GlobalContext<'gcx>, _: &UnitPath) -> Self {
        TyCtx {
            global_ctx,
            locals: HashMap::new(),
        }
    }

    pub fn add_local(&mut self, hil_id: HilId, t: TypeRef<'gcx>) {
        self.locals.insert(hil_id, t);
    }

    pub fn parse_ty(&mut self, node: &Node) -> TypeRef<'gcx> {
        self.global_ctx.interned_type(node)
    }

    pub fn unit_type(&self) -> TypeRef<'gcx> {
        &self.global_ctx.unit_type
    }

    pub fn eq_type(&mut self, t: TypeRef<'gcx>, u: TypeRef<'gcx>) -> bool {
        match (t.kind(), u.kind()) {
            (TypeKind::Adt(adt_def_t, _), TypeKind::Adt(adt_def_u, _)) => {
                return adt_def_t.def_id == adt_def_u.def_id;
            }
            _ => panic!(),
        }
    }

    pub fn sub_type(&mut self, t: TypeRef<'gcx>, u: TypeRef<'gcx>) -> bool {
        self.eq_type(t, u)
    }

    pub fn check_expr(&mut self, dst: TypeRef<'gcx>, expr: &Node) {
        assert_eq!(expr.car().as_symbol().unwrap(), "Expr");

        let src = self.infer_expr(expr);
        if !self.sub_type(dst, src) {
            panic!()
        }
    }

    pub fn infer_expr(&mut self, expr: &Node) -> TypeRef<'gcx> {
        assert_eq!(expr.car().as_symbol().unwrap(), "Expr");
        let kind = node_get_attr(expr, "kind").as_keyword().unwrap();
        if kind == "qpath" {
            let qpath = node_get_field(expr, 0).as_cons().unwrap();
            assert_eq!(qpath.car().as_symbol().unwrap(), "Path");
            let kind = node_get_attr(qpath, "kind").as_keyword().unwrap();
            if kind == "local" {
                let hil_id = HilId::from_s_expr(node_get_attr(qpath, "local"));
                return self.locals.get(&hil_id).unwrap();
            } else {
                panic!("Cannot infer type for Path kind {}", kind);
            }
        } else {
            panic!("Cannot infer type for Expr kind {}", kind);
        }
    }
}
