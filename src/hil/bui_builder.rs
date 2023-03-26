use capnp::message::TypedBuilder;

use crate::bui::BuiMessage;
use crate::bui_capnp;
use crate::context::GlobalContext;
use crate::hil::{DefId, HilVisitor};
use crate::s_expr::{node_get_attr, node_get_field, node_get_fields};

pub struct ToBuiContext<'gcx> {
    ctx: &'gcx GlobalContext<'gcx>,
}

pub struct ToBuiVisitor<'gcx> {
    builder: TypedBuilder<bui_capnp::unit::Owned>,
    namespace: Vec<String>,
    item_num: u32,
    ctx: ToBuiContext<'gcx>,
}

impl<'gcx> ToBuiVisitor<'gcx> {
    pub fn new(item_num: usize, ctx: &'gcx GlobalContext<'gcx>) -> Self {
        let mut builder = TypedBuilder::<bui_capnp::unit::Owned>::new_default();
        let root = builder.init_root();
        root.init_items(u32::try_from(item_num).unwrap());
        ToBuiVisitor {
            builder,
            namespace: vec![],
            item_num: 0,
            ctx: ToBuiContext { ctx },
        }
    }

    pub fn collect(self) -> BuiMessage {
        BuiMessage::new(&self.builder)
    }
}

fn set_ty(builder: &mut bui_capnp::ty::Builder, node: &crate::hil::Node, ctx: &ToBuiContext) {
    let kind = node_get_attr(node, "kind").as_keyword().unwrap();
    if kind == "bool" {
        builder.set_bool(());
    } else if kind == "int" {
        builder.set_int(());
    } else if kind == "qpath" {
        let qpath = node_get_field(node, 0).as_cons().unwrap();
        let def_id = DefId::from_s_expr(node_get_field(qpath, 0));
        let mut adt_builder = builder.reborrow().init_adt();

        def_id.serialize(&mut adt_builder);
    } else if kind == "tuple" {
        let fields = node_get_field(node, 0).as_slice().unwrap();
        let mut fields_builder = builder
            .reborrow()
            .init_tuple(u32::try_from(fields.len()).unwrap());
        for (i, field) in fields.iter().enumerate() {
            let mut ty_builder = fields_builder.reborrow().get(u32::try_from(i).unwrap());
            set_ty(&mut ty_builder, field.as_cons().unwrap(), ctx);
        }
    }
}

fn set_defn(
    defn_builder: &mut bui_capnp::defn::Builder,
    node: &crate::hil::Node,
    ctx: &ToBuiContext,
) {
    let fn_sig_builder = defn_builder.reborrow().init_fn_sig();
    let fn_decl_builder = fn_sig_builder.init_decl();
    let inputs = node_get_field(
        node_get_field(node_get_field(node, 1).as_cons().unwrap(), 0)
            .as_cons()
            .unwrap(),
        0,
    )
    .as_slice()
    .unwrap();
    let mut inputs_builder = fn_decl_builder.init_inputs(u32::try_from(inputs.len()).unwrap());
    for (i, input) in inputs.iter().enumerate() {
        let mut ty_builder = inputs_builder.reborrow().get(u32::try_from(i).unwrap());
        set_ty(
            &mut ty_builder,
            node_get_field(input.as_cons().unwrap(), 1)
                .as_cons()
                .unwrap(),
            ctx,
        );
    }
}

impl<'gcx> HilVisitor for ToBuiVisitor<'gcx> {
    fn visit_item(&mut self, node: &super::Node) {
        let kind = node_get_attr(node, "kind").as_keyword().unwrap();
        let root = self.builder.get_root().unwrap();
        let mut builder = root.get_items().unwrap().get(self.item_num);
        let ident = node_get_attr(node, "ident").as_symbol().unwrap();
        builder.set_ident(ident);
        let mut namespace_builder = builder
            .reborrow()
            .init_namespace(u32::try_from(self.namespace.len()).unwrap());
        for (i, name) in self.namespace.iter().enumerate() {
            namespace_builder.set(u32::try_from(i).unwrap(), name);
        }
        let def_id = node_get_attr(node, "def_id").as_u64().unwrap();
        builder.reborrow().set_def(u32::try_from(def_id).unwrap());
        let kind_builder = builder.reborrow().init_kind();
        if kind == "class" {
            let class_builder = kind_builder.init_class();
            class_builder.init_fields(0);
        } else if kind == "defn" {
            set_defn(&mut kind_builder.init_defn().reborrow(), node, &self.ctx);
        } else if kind == "interface" {
            let mut interface_builder = kind_builder.init_interface();
            let decls = node_get_fields(node_get_field(node, 0).as_cons().unwrap());
            let types = decls
                .iter()
                .filter(|&x| {
                    node_get_attr(x.as_cons().unwrap(), "kind")
                        .as_keyword()
                        .unwrap()
                        == "ty"
                })
                .collect::<Vec<_>>();
            let mut types_builder = interface_builder
                .reborrow()
                .init_types(u32::try_from(types.len()).unwrap());
            for (i, ty) in types.iter().enumerate() {
                let mut ty_builder = types_builder.reborrow().get(u32::try_from(i).unwrap());
                ty_builder
                    .set_key(
                        node_get_attr(ty.as_cons().unwrap(), "ident")
                            .as_symbol()
                            .unwrap(),
                    )
                    .unwrap();
                let components = node_get_fields(ty.as_cons().unwrap());
                if components.len() == 0 {
                    ty_builder.init_value().set_opaque(());
                } else {
                    set_ty(
                        &mut ty_builder.init_value().init_transparent(),
                        components[0].as_cons().unwrap(),
                        &self.ctx,
                    );
                }
            }
            let defns = decls
                .iter()
                .filter(|&x| {
                    node_get_attr(x.as_cons().unwrap(), "kind")
                        .as_keyword()
                        .unwrap()
                        == "defn"
                })
                .collect::<Vec<_>>();
            let mut defns_builder = interface_builder
                .reborrow()
                .init_defns(u32::try_from(defns.len()).unwrap());
            for (i, defn) in defns.iter().enumerate() {
                let mut defn_builder = defns_builder.reborrow().get(u32::try_from(i).unwrap());
                defn_builder
                    .set_key(
                        node_get_attr(defn.as_cons().unwrap(), "ident")
                            .as_symbol()
                            .unwrap(),
                    )
                    .unwrap();
                set_defn(
                    &mut defn_builder.init_value(),
                    defn.as_cons().unwrap(),
                    &self.ctx,
                );
            }
        } else if kind == "module" {
            let mut module_builder = kind_builder.init_module();
            module_builder.reborrow().init_modules(0);
            let bindings = node_get_fields(node_get_field(node, 0).as_cons().unwrap());
            let types = bindings
                .iter()
                .filter(|&x| {
                    node_get_attr(x.as_cons().unwrap(), "kind")
                        .as_keyword()
                        .unwrap()
                        == "ty"
                })
                .collect::<Vec<_>>();
            let mut types_builder = module_builder
                .reborrow()
                .init_types(u32::try_from(types.len()).unwrap());
            for (i, ty) in types.iter().enumerate() {
                let mut ty_builder = types_builder.reborrow().get(u32::try_from(i).unwrap());
                ty_builder
                    .set_key(
                        node_get_attr(ty.as_cons().unwrap(), "ident")
                            .as_symbol()
                            .unwrap(),
                    )
                    .unwrap();
                let t = node_get_field(ty.as_cons().unwrap(), 0);
                set_ty(
                    &mut ty_builder.init_value(),
                    t.as_cons().unwrap(),
                    &self.ctx,
                );
            }
            let defns = bindings
                .iter()
                .filter(|&x| {
                    node_get_attr(x.as_cons().unwrap(), "kind")
                        .as_keyword()
                        .unwrap()
                        == "defn"
                })
                .collect::<Vec<_>>();
            let mut defns_builder = module_builder
                .reborrow()
                .init_defns(u32::try_from(defns.len()).unwrap());
            for (i, defn) in defns.iter().enumerate() {
                let mut defn_builder = defns_builder.reborrow().get(u32::try_from(i).unwrap());
                defn_builder
                    .set_key(
                        node_get_attr(defn.as_cons().unwrap(), "ident")
                            .as_symbol()
                            .unwrap(),
                    )
                    .unwrap();
                set_defn(
                    &mut defn_builder.init_value(),
                    defn.as_cons().unwrap(),
                    &self.ctx,
                );
            }
        }
        self.item_num += 1;
    }

    fn visit_pre_namespace(&mut self, node: &super::Node) {
        let n = node_get_field(node, 0).as_symbol().unwrap();
        self.namespace.push(n.to_string());
    }

    fn visit_post_namespace(&mut self, _: &super::Node) {
        self.namespace.pop();
    }
}
