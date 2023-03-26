pub mod context;

use std::collections::HashMap;

use crate::context::GlobalContext;
use crate::hil::{
    hil_get_unit_path, walk_hil_node, Hil, HilId, HilVisitor, Node,
};
use crate::def::{DefId, DefLocalId};
use crate::s_expr::{node_get_attr, node_get_field};

use context::{TyCtx, UnitPath};

pub type TypeRef<'gcx> = &'gcx Type<'gcx>;

pub enum GenericArg<'gcx> {
    Ty(TypeRef<'gcx>),
}

pub struct FieldDef {
    pub def_id: DefId,
    pub name: String,
}

pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

pub struct AdtDef {
    pub def_id: DefId,
    variants: Vec<VariantDef>,
}

impl AdtDef {
    pub fn new(def_id: DefId) -> Self {
        AdtDef {
            def_id,
            variants: vec![],
        }
    }
}

pub enum TypeKind<'gcx> {
    Bool,
    Adt(&'gcx AdtDef, &'gcx [GenericArg<'gcx>]),
    Tuple(&'gcx [TypeRef<'gcx>]),
}

pub struct Type<'gcx> {
    kind: TypeKind<'gcx>,
}

impl<'gcx> Type<'gcx> {
    pub fn make_adt(def: &'gcx AdtDef) -> Self {
        Type {
            kind: TypeKind::Adt(def, &[]),
        }
    }

    pub fn make_tuple(elems: &'gcx [TypeRef<'gcx>]) -> Self {
        Type {
            kind: TypeKind::Tuple(elems),
        }
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

pub struct TypeCheckContext<'gcx> {
    ty_ctxs: HashMap<DefId, TyCtx<'gcx>>,
}

impl<'gcx> TypeCheckContext<'gcx> {
    fn new() -> Self {
        TypeCheckContext {
            ty_ctxs: HashMap::new(),
        }
    }
}

pub struct UnitTypeChecker<'gcx> {
    global_ctx: &'gcx GlobalContext<'gcx>,
    unit_path: UnitPath,
    ctx: TypeCheckContext<'gcx>,
    item_num: usize,
}

impl<'gcx> UnitTypeChecker<'gcx> {
    pub fn new(
        global_ctx: &'gcx GlobalContext<'gcx>,
        unit_path: UnitPath,
    ) -> Self {
        UnitTypeChecker {
            global_ctx,
            unit_path,
            ctx: TypeCheckContext::new(),
            item_num: 0,
        }
    }

    fn check_defn(&mut self, node: &Node) {
        let ty_ctx = TyCtx::new(self.global_ctx, &self.unit_path);
        let mut defn_type_checker = DefnTypeChecker::new(ty_ctx);
        walk_hil_node(&mut defn_type_checker, node);
        let def_local_id = DefLocalId::from_s_expr(node_get_attr(node, "def_id"));
        self.ctx.ty_ctxs.insert(
            def_local_id.to_def_id(),
            defn_type_checker.collect(),
        );
    }

    fn collect(self) -> TypeCheckContext<'gcx> {
        self.ctx
    }
}

impl<'a> HilVisitor for UnitTypeChecker<'a> {
    fn visit_item(&mut self, node: &Node) {
        let kind = node_get_attr(node, "kind").as_keyword().unwrap();
        if kind == "defn" {
            self.check_defn(node);
        }
        self.item_num += 1;
    }

    fn visit_binding(&mut self, node: &Node) {
        let kind = node_get_attr(node, "kind").as_keyword().unwrap();
        if kind == "defn" {
            self.check_defn(node);
        }
    }
}

pub struct DefnTypeChecker<'gcx> {
    ty_ctx: TyCtx<'gcx>,
    block_types: Vec<TypeRef<'gcx>>,
}

impl<'gcx> DefnTypeChecker<'gcx> {
    fn new(ty_ctx: TyCtx<'gcx>) -> Self {
        DefnTypeChecker {
            ty_ctx,
            block_types: vec![],
        }
    }

    fn get_block_type(&self) -> TypeRef<'gcx> {
        self.block_types.last().unwrap()
    }

    fn collect(self) -> TyCtx<'gcx> {
        self.ty_ctx
    }
}

impl<'gcx> HilVisitor for DefnTypeChecker<'gcx> {
    fn visit_fn_sig(&mut self, node: &Node) {
        let decl = node_get_field(node, 0).as_cons().unwrap();
        let params = node_get_field(decl, 0).as_slice().unwrap();
        params.iter().for_each(|x| {
            let n = x.as_cons().unwrap();
            let hil_id = HilId::from_s_expr(node_get_attr(n, "hil_id"));
            let t = self
                .ty_ctx
                .parse_ty(node_get_field(n, 1).as_cons().unwrap());
            self.ty_ctx.add_local(hil_id, t);
        });
        let fn_ret_ty = node_get_field(decl, 1).as_cons().unwrap();
        self.block_types.push(
            self.ty_ctx
                .parse_ty(node_get_field(fn_ret_ty, 0).as_cons().unwrap()),
        );
    }

    fn visit_stmt(&mut self, node: &Node) {
        let kind = node_get_attr(node, "kind").as_keyword().unwrap();
        if kind == "expr" {
            self.ty_ctx.check_expr(
                self.get_block_type(),
                node_get_field(node, 0).as_cons().unwrap(),
            );
        }
    }

    fn visit_post_block(&mut self, _: &Node) {
        self.block_types.pop();
    }
}

pub fn ty_check<'gcx>(global_ctx: &'gcx GlobalContext<'gcx>, hil: &Hil) -> TypeCheckContext<'gcx> {
    let mut type_checker = UnitTypeChecker::new(
        global_ctx,
        hil_get_unit_path(hil),
    );
    type_checker.visit(hil);
    type_checker.collect()
}
