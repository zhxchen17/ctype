use std::collections::HashMap;
use std::cell::RefCell;

use typed_arena::Arena;

use crate::bui::Bui;
use crate::def::DefId;
use crate::hil::Node;
use crate::s_expr::{node_get_attr, node_get_field};
use crate::t::{Type, AdtDef};
use crate::t::context::UnitPath;

struct UnitCache {
    units: Vec<Bui>,
    index: HashMap<UnitPath, usize>,
}

impl UnitCache {
    fn new() -> Self {
        UnitCache {
            units: vec![],
            index: HashMap::new(),
        }
    }

    fn load(&mut self, unit_path: &UnitPath) {
        self.index.insert(unit_path.clone(), self.units.len());
        self.units.push(Bui::empty());
    }

}

struct TypeCache<'gcx> {
    types: Arena<Type<'gcx>>,
    adt_defs: Arena<AdtDef>,
}

impl<'gcx> TypeCache<'gcx> {
    fn new() -> Self {
        TypeCache {
            types: Arena::new(),
            adt_defs: Arena::new(),
        }
    }

    fn adt_def(&self, def_id: DefId) -> &AdtDef {
        &*self.adt_defs.alloc(AdtDef::new(def_id))
    }

    fn adt(&self, adt_def: &'gcx AdtDef) -> &Type<'gcx> {
        &*self.types.alloc(Type::make_adt(adt_def))
    }
}

pub struct GlobalContext<'gcx> {
    unit_cache: RefCell<UnitCache>,
    type_cache: TypeCache<'gcx>,
    pub unit_type: Type<'gcx>,
}

impl<'gcx> GlobalContext<'gcx> {
    pub fn new() -> Self {
        GlobalContext {
            unit_cache: RefCell::new(UnitCache::new()),
            type_cache: TypeCache::new(),
            unit_type: Type::make_tuple(&[]),
        }
    }

    pub fn load_unit(&self, unit_path: &UnitPath) {
        self.unit_cache.borrow_mut().load(unit_path);
    }

    pub fn interned_type(&'gcx self, node: &Node) -> &'gcx Type<'gcx> {
        assert_eq!(node.car().as_symbol().unwrap(), "Ty");
        let kind = node_get_attr(node, "kind").as_keyword().unwrap();
        if kind == "qpath" {
            let path = node_get_field(node, 0).as_cons().unwrap();
            let def_id = DefId::from_s_expr(node_get_field(path, 0));
            let adt_def = self.type_cache.adt_def(def_id);
            self.type_cache.adt(adt_def)
        } else {
            panic!("unsupported type: {}", kind);
        }
    }
}
