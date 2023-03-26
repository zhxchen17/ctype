use std::collections::HashMap;

use lexpr::{sexp, Cons, Value};

use crate::ast::{Ast, AstVisitor, Node, NodeId};
use crate::def::{DefId, DefLocalId};
use crate::error::{report_error, Error};
use crate::hil::{Hil, HilId};
use crate::s_expr::{node_add_attr, node_get_attr, node_get_field};

pub enum Resolution<Id> {
    Def(DefId),
    Local(Id),
}

#[derive(Copy, Clone)]
struct NamespaceId {
    private: usize,
}

impl NamespaceId {
    fn new(private: usize) -> Self {
        NamespaceId { private }
    }
}

struct Namespace {
    parent: NamespaceId,
    children: HashMap<String, NamespaceId>,
    defs: HashMap<String, DefId>,
}

impl Namespace {
    fn new(parent: NamespaceId) -> Self {
        Namespace {
            parent,
            children: HashMap::new(),
            defs: HashMap::new(),
        }
    }

    fn def(&mut self, ident: String, def_id: DefId) {
        self.defs.insert(ident, def_id);
    }

    fn lookup(&self, ident: &str) -> Option<DefId> {
        self.defs.get(ident).cloned()
    }
}

pub struct NamespaceContext {
    namespaces: Vec<Namespace>,
    current_ns: NamespaceId,
}

impl NamespaceContext {
    fn make_namespace(&mut self, name: String) -> NamespaceId {
        let namespace_id = NamespaceId::new(self.namespaces.len());
        self.namespaces.push(Namespace::new(self.current_ns));
        self.get_namespace().children.insert(name, namespace_id);
        namespace_id
    }

    fn get_namespace(&mut self) -> &mut Namespace {
        &mut self.namespaces[self.current_ns.private]
    }

    fn new() -> Self {
        let mut ret = NamespaceContext {
            namespaces: vec![],
            current_ns: NamespaceId::new(0),
        };
        ret.make_namespace(String::new());
        ret
    }
}

trait NamespaceManager {
    fn get_namespace_context(&mut self) -> &mut NamespaceContext;

    fn enter_namespace(&mut self, name: String) {
        let ctx = self.get_namespace_context();
        let children = &ctx.get_namespace().children;
        if let Some(child) = children.get(&name) {
            ctx.current_ns = *child;
        } else {
            ctx.current_ns = ctx.make_namespace(name);
        }
    }

    fn exit_namespace(&mut self) {
        let ctx = self.get_namespace_context();
        ctx.current_ns = ctx.get_namespace().parent;
    }
}

pub struct ResolveCollectVisitor {
    ns_ctx: NamespaceContext,
    def_local_id: DefLocalId,
}

impl ResolveCollectVisitor {
    pub fn new() -> Self {
        ResolveCollectVisitor {
            ns_ctx: NamespaceContext::new(),
            def_local_id: DefLocalId::new(),
        }
    }

    pub fn collect(self) -> NamespaceContext {
        self.ns_ctx
    }
}

impl NamespaceManager for ResolveCollectVisitor {
    fn get_namespace_context(&mut self) -> &mut NamespaceContext {
        &mut self.ns_ctx
    }
}

impl AstVisitor for ResolveCollectVisitor {
    fn visit_item(&mut self, node: &Node) {
        let ident = node_get_attr(node, "ident");
        let id = self.def_local_id;
        self.def_local_id = id.next();
        self.get_namespace_context()
            .get_namespace()
            .def(ident.to_string(), id.to_def_id());
    }

    fn visit_pre_namespace(&mut self, node: &Node) {
        let name = node_get_field(node, 0);
        self.enter_namespace(name.to_string());
    }

    fn visit_post_namespace(&mut self, _: &Node) {
        self.exit_namespace();
    }
}

pub struct ResolvePathVisitor {
    ns_ctx: NamespaceContext,
    locals: HashMap<String, NodeId>,
    resolutions: HashMap<NodeId, Resolution<NodeId>>,
}

impl ResolvePathVisitor {
    pub fn new(ns_ctx: NamespaceContext) -> Self {
        ResolvePathVisitor {
            ns_ctx,
            locals: HashMap::new(),
            resolutions: HashMap::new(),
        }
    }

    pub fn resolve(self) -> HashMap<NodeId, Resolution<NodeId>> {
        self.resolutions
    }
}

impl NamespaceManager for ResolvePathVisitor {
    fn get_namespace_context(&mut self) -> &mut NamespaceContext {
        &mut self.ns_ctx
    }
}

impl AstVisitor for ResolvePathVisitor {
    fn visit_param(&mut self, node: &Node) {
        let ident = node_get_field(node, 0);
        self.locals
            .insert(ident.as_symbol().unwrap().to_string(), NodeId::new(node));
    }

    fn visit_expr(&mut self, node: &Node) {
        let kind = node_get_attr(node, "kind");
        if kind.as_keyword().unwrap() != "path" {
            return;
        }

        let segments = node_get_field(node, 0).as_slice().unwrap();
        if segments.len() > 1 {
            panic!("qualified path is not yet implemented.")
        }
        let ident = segments[0].as_symbol().unwrap();
        if let Some(node_id) = self.locals.get(ident) {
            self.resolutions
                .insert(NodeId::new(node), Resolution::Local(*node_id));
            return;
        }
        if let Some(def_id) = self.get_namespace_context().get_namespace().lookup(ident) {
            self.resolutions
                .insert(NodeId::new(node), Resolution::Def(def_id));
        } else {
            report_error(Error::UndefinedName(ident.to_string()));
        }
    }

    fn visit_type(&mut self, node: &Node) {
        let kind = node_get_attr(node, "kind");
        if kind.as_keyword().unwrap() != "path" {
            return;
        }

        let segments = node_get_field(node, 0).as_slice().unwrap();
        if segments.len() > 1 {
            panic!("qualified path is not yet implemented.")
        }
        let ident = segments[0].as_symbol().unwrap();
        if let Some(def_id) = self.get_namespace_context().get_namespace().lookup(ident) {
            self.resolutions
                .insert(NodeId::new(node), Resolution::Def(def_id));
        } else {
            report_error(Error::UndefinedName(ident.to_string()));
        }
    }

    fn visit_pre_namespace(&mut self, node: &Node) {
        let name = node_get_field(node, 0);
        self.enter_namespace(name.to_string());
    }

    fn visit_post_namespace(&mut self, _: &Node) {
        self.exit_namespace();
    }
}

pub struct LoweringContext {
    resolutions: HashMap<NodeId, Resolution<NodeId>>,
    node_to_hil: HashMap<NodeId, HilId>,
}

impl LoweringContext {
    fn hil_id(&mut self, node_id: NodeId) -> HilId {
        *self
            .node_to_hil
            .entry(node_id)
            .or_insert_with(|| HilId::new())
    }
    pub fn new(resolutions: HashMap<NodeId, Resolution<NodeId>>) -> Self {
        LoweringContext {
            resolutions,
            node_to_hil: HashMap::new(),
        }
    }
}

struct AstLowering {
    def_local_id: DefLocalId,
}

impl AstLowering {
    fn run(&mut self, ast: &Ast, ctx: &mut LoweringContext) -> Hil {
        match ast {
            Value::Cons(c) => {
                let (car, cdr) = c.as_pair();

                let head = car.as_symbol().unwrap().to_string();
                let def_local_id = if head == "Item"
                    || head == "Decl"
                    || head == "Binding"
                    || head == "Variant"
                    || head == "FieldDef"
                {
                    let d = Some(self.def_local_id);
                    self.def_local_id = self.def_local_id.next();
                    d
                } else {
                    None
                };
                let mut hil = if (head == "Expr" || head == "Ty")
                    && node_get_attr(c, "kind").as_keyword() == Some("path")
                {
                    let resolved =
                        |k, r| Cons::new(car.clone(), sexp!((#:kind #:qpath (Path #:kind ,k ,r))));
                    match ctx.resolutions.get(&NodeId::new(c)).unwrap() {
                        Resolution::Def(def_id) => {
                            resolved(Value::keyword("def"), def_id.to_s_expr())
                        }
                        Resolution::Local(node_id) => {
                            resolved(Value::keyword("local"), ctx.hil_id(*node_id).to_s_expr())
                        }
                    }
                } else {
                    Cons::new(
                        self.run(car, ctx),
                        Value::list(
                            cdr.list_iter()
                                .unwrap()
                                .map(|x| self.run(x, ctx))
                                .collect::<Vec<Hil>>(),
                        ),
                    )
                };

                if def_local_id.is_some() {
                    hil = node_add_attr(hil, "def_id", def_local_id.unwrap().to_s_expr());
                }
                if head != "Item" && head != "Unit" {
                    hil = node_add_attr(hil, "hil_id", ctx.hil_id(NodeId::new(c)).to_s_expr());
                }
                Value::Cons(hil)
            }
            Value::Vector(v) => {
                Value::vector(v.iter().map(|x| self.run(x, ctx)).collect::<Vec<Hil>>())
            }
            x => x.clone(),
        }
    }
}

pub fn lower_ast(ast: &Ast, ctx: &mut LoweringContext) -> Hil {
    let mut lowering = AstLowering {
        def_local_id: DefLocalId::new(),
    };
    lowering.run(ast, ctx)
}
