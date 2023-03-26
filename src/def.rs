use lexpr::{sexp, Value};

use crate::s_expr::node_get_field;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UnitNum(u16);

const LOCAL_UNIT: u16 = 0;

impl UnitNum {
    fn new(x: u16) -> Self {
        UnitNum { 0: x }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DefLocalId(u32);

impl DefLocalId {
    pub fn new() -> Self {
        DefLocalId(0)
    }

    pub fn next(&self) -> DefLocalId {
        DefLocalId(self.0 + 1)
    }

    pub fn to_def_id(&self) -> DefId {
        DefId {
            unit: UnitNum::new(LOCAL_UNIT),
            offset: *self,
        }
    }

    pub fn from_s_expr(value: &Value) -> Self {
        DefLocalId(u32::try_from(value.as_u64().unwrap()).unwrap())
    }

    pub fn to_s_expr(&self) -> Value {
        let def = self.0;
        sexp!(,def)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

    pub fn local(&self) -> DefLocalId {
        self.offset
    }

    pub fn serialize(&self, builder: &mut crate::bui_capnp::item_ref::Builder) {
        builder.set_unit(self.unit.0);
        builder.set_def(self.offset.0)
    }
}

