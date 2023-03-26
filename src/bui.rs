use capnp::message::{ReaderOptions, TypedBuilder, TypedReader};
use capnp::serialize::{read_message, write_message_to_words, OwnedSegments};

use crate::bui_capnp;

pub struct BuiMessage {
    buffer: Vec<u8>,
}

impl BuiMessage {
    pub fn new(builder: &TypedBuilder<bui_capnp::unit::Owned>) -> Self {
        BuiMessage {
            buffer: write_message_to_words(builder.borrow_inner()),
        }
    }

    pub fn deserialize(&self) -> Bui {
        let reader = read_message(self.buffer.as_slice(), ReaderOptions::new()).unwrap();
        Bui {
            reader: Some(TypedReader::<_, bui_capnp::unit::Owned>::new(reader)),
        }
    }
}

pub struct BuiKind<'a> {
    reader: bui_capnp::kind::Reader<'a>,
}

impl<'a> BuiKind<'a> {
    pub fn is_ty(&self) -> bool {
        if let Ok(ok) = self.reader.which() {
            if let bui_capnp::kind::Ty(_) = ok {
                return true;
            }
        }
        false
    }
}

pub struct BuiTyDecl<'a> {
    reader: bui_capnp::ty_decl::Reader<'a>,
}

impl<'a> BuiTyDecl<'a> {
    pub fn transparent(&self) -> Option<BuiTy> {
        if let Ok(ok) = self.reader.which() {
            if let bui_capnp::ty_decl::Transparent(t) = ok {
                return Some(BuiTy { reader: t.unwrap() });
            }
        }
        None
    }
}

pub struct BuiInterface<'a> {
    reader: bui_capnp::interface::Reader<'a>,
}

impl<'a> BuiInterface<'a> {
    pub fn types(&self) -> Vec<(&str, BuiTyDecl)> {
        self.reader
            .get_types()
            .unwrap()
            .iter()
            .map(|x| {
                (
                    x.get_key().unwrap(),
                    BuiTyDecl {
                        reader: x.get_value().unwrap(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn defns(&self) -> Vec<(&str, BuiDefn)> {
        self.reader
            .get_defns()
            .unwrap()
            .iter()
            .map(|x| {
                (
                    x.get_key().unwrap(),
                    BuiDefn {
                        reader: x.get_value().unwrap(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }
}

pub struct BuiFieldDef<'a> {
    reader: bui_capnp::field_def::Reader<'a>,
}

impl<'a> BuiFieldDef<'a> {
    pub fn name(&self) -> &str {
        self.reader.get_name().unwrap()
    }
}

pub struct BuiClass<'a> {
    reader: bui_capnp::class::Reader<'a>,
}

impl<'a> BuiClass<'a> {
    pub fn fields(&self) -> Vec<BuiFieldDef> {
        self.reader
            .get_fields()
            .unwrap()
            .iter()
            .map(|x| BuiFieldDef { reader: x })
            .collect::<Vec<_>>()
    }
}

pub struct BuiItemRef<'a> {
    reader: bui_capnp::item_ref::Reader<'a>,
}

impl<'a> BuiItemRef<'a> {
    pub fn unit(&self) -> u16 {
        self.reader.get_unit()
    }

    pub fn def(&self) -> u32 {
        self.reader.get_def()
    }
}

pub struct BuiTy<'a> {
    reader: bui_capnp::ty::Reader<'a>,
}

impl<'a> BuiTy<'a> {
    pub fn is_bool(&self) -> bool {
        if let Ok(ok) = self.reader.which() {
            if let bui_capnp::ty::Bool(_) = ok {
                return true;
            }
        }
        false
    }

    pub fn adt(&self) -> Option<BuiItemRef> {
        if let Ok(ok) = self.reader.which() {
            if let bui_capnp::ty::Adt(a) = ok {
                return Some(BuiItemRef { reader: a.unwrap() });
            }
        }
        None
    }

    pub fn tuple(&self) -> Option<Vec<BuiTy>> {
        if let Ok(ok) = self.reader.which() {
            if let bui_capnp::ty::Tuple(t) = ok {
                return Some(
                    t.unwrap()
                        .iter()
                        .map(|x| BuiTy { reader: x })
                        .collect::<Vec<_>>(),
                );
            }
        }
        None
    }
}

pub struct BuiFnDecl<'a> {
    reader: bui_capnp::fn_decl::Reader<'a>,
}

impl<'a> BuiFnDecl<'a> {
    pub fn inputs(&self) -> Vec<BuiTy> {
        self.reader
            .get_inputs()
            .unwrap()
            .iter()
            .map(|x| BuiTy { reader: x })
            .collect::<Vec<_>>()
    }
}

pub struct BuiFnSig<'a> {
    reader: bui_capnp::fn_sig::Reader<'a>,
}

impl<'a> BuiFnSig<'a> {
    pub fn decl(&self) -> BuiFnDecl {
        BuiFnDecl {
            reader: self.reader.get_decl().unwrap(),
        }
    }
}

pub struct BuiDefn<'a> {
    reader: bui_capnp::defn::Reader<'a>,
}

impl<'a> BuiDefn<'a> {
    pub fn fn_sig(&self) -> BuiFnSig {
        BuiFnSig {
            reader: self.reader.get_fn_sig().unwrap(),
        }
    }
}

pub struct BuiModule<'a> {
    reader: bui_capnp::module::Reader<'a>,
}

impl<'a> BuiModule<'a> {
    pub fn types(&self) -> Vec<(&str, BuiTy)> {
        self.reader
            .get_types()
            .unwrap()
            .iter()
            .map(|x| {
                (
                    x.get_key().unwrap(),
                    BuiTy {
                        reader: x.get_value().unwrap(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn defns(&self) -> Vec<(&str, BuiDefn)> {
        self.reader
            .get_defns()
            .unwrap()
            .iter()
            .map(|x| {
                (
                    x.get_key().unwrap(),
                    BuiDefn {
                        reader: x.get_value().unwrap(),
                    },
                )
            })
            .collect::<Vec<_>>()
    }
}

pub struct BuiItem<'a> {
    reader: bui_capnp::item::Reader<'a>,
}

impl<'a> BuiItem<'a> {
    pub fn ident(&self) -> &str {
        self.reader.get_ident().unwrap()
    }
    pub fn namespace(&self) -> Vec<&str> {
        self.reader
            .get_namespace()
            .unwrap()
            .iter()
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
    }
    pub fn interface(&self) -> Option<BuiInterface> {
        if let Ok(ok) = self.reader.get_kind().which() {
            if let bui_capnp::item::kind::Interface(i) = ok {
                return Some(BuiInterface { reader: i.unwrap() });
            }
        }
        None
    }
    pub fn class(&self) -> Option<BuiClass> {
        if let Ok(ok) = self.reader.get_kind().which() {
            if let bui_capnp::item::kind::Class(c) = ok {
                return Some(BuiClass { reader: c.unwrap() });
            }
        }
        None
    }
    pub fn defn(&self) -> Option<BuiDefn> {
        if let Ok(ok) = self.reader.get_kind().which() {
            if let bui_capnp::item::kind::Defn(d) = ok {
                return Some(BuiDefn { reader: d.unwrap() });
            }
        }
        None
    }
    pub fn module(&self) -> Option<BuiModule> {
        if let Ok(ok) = self.reader.get_kind().which() {
            if let bui_capnp::item::kind::Module(m) = ok {
                return Some(BuiModule { reader: m.unwrap() });
            }
        }
        None
    }
}

pub struct Bui {
    reader: Option<TypedReader<OwnedSegments, bui_capnp::unit::Owned>>,
}

impl Bui {
    pub fn empty() -> Self {
        Bui { reader: None }
    }

    pub fn items(&self) -> Vec<BuiItem> {
        let items = self
            .reader
            .as_ref()
            .unwrap()
            .get()
            .unwrap()
            .get_items()
            .unwrap()
            .iter()
            .map(|i| BuiItem { reader: i })
            .collect::<Vec<_>>();
        items
    }
}
