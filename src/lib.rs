pub mod ast;
pub mod bui;
pub mod bui_capnp {
    include!(concat!(env!("OUT_DIR"), "/bui_capnp.rs"));
}
pub mod context;
mod def;
mod error;
pub mod hil;
mod s_expr;
pub mod t;
pub mod til;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lexpr::{sexp, Value};

    struct MatchContext {
        vars: HashMap<u32, Value>,
    }

    impl MatchContext {
        fn new() -> Self {
            MatchContext {
                vars: HashMap::new(),
            }
        }
    }

    fn sexp_match(a: &Value, b: &Value, ctx: &mut MatchContext) -> bool {
        if let Some(k) = a.as_keyword() {
            if k.starts_with("_") {
                panic!(
                    "Keywords start with _ is preserved in intermediate languages, got: {}",
                    k
                );
            }
        }
        if let Some(k) = b.as_keyword() {
            if k.starts_with("_") {
                if k.len() == 1 {
                    return true;
                } else if let Ok(i) = k[1..].parse::<u32>() {
                    if let Some(v) = ctx.vars.get(&i) {
                        return v == a;
                    } else {
                        ctx.vars.insert(i, a.clone());
                        return true;
                    }
                }
            }
        }
        match (a, b) {
            (Value::Nil, Value::Nil) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Symbol(x), Value::Symbol(y)) => x == y,
            (Value::Keyword(x), Value::Keyword(y)) => x == y,
            (Value::Bytes(x), Value::Bytes(y)) => x == y,
            (Value::Cons(x), Value::Cons(y)) => {
                sexp_match(x.car(), y.car(), ctx) && sexp_match(x.cdr(), y.cdr(), ctx)
            }
            (Value::Vector(x), Value::Vector(y)) => {
                if x.len() != y.len() {
                    false
                } else {
                    x.iter().zip(y.iter()).all(|(i, j)| sexp_match(i, j, ctx))
                }
            }
            _ => false,
        }
    }

    #[test]
    fn test_sexp_match() {
        let a = sexp!((1 (2 3)));
        let b = sexp!((1 (3 3)));
        assert!(!sexp_match(&a, &b, &mut MatchContext::new()));

        let a = sexp!((1 (2 3)));
        let b = sexp!((1 (2 3)));
        assert!(sexp_match(&a, &b, &mut MatchContext::new()));

        let a = sexp!((1 (2 #:a)));
        let b = sexp!((1 (2 #:_)));
        assert!(sexp_match(&a, &b, &mut MatchContext::new()));

        let a = sexp!((1 (2 1)));
        let b = sexp!((#:_1 (2 #:_1)));
        assert!(sexp_match(&a, &b, &mut MatchContext::new()));

        let a = sexp!((1 (2 3)));
        let b = sexp!((#:_1 (2 #:_1)));
        assert!(!sexp_match(&a, &b, &mut MatchContext::new()));
    }

    fn get_ast() -> Value {
        // class Bar {}
        // defn foo(x: Bar, y: Bar) -> Bar {
        //   x;
        //   y
        // }
        // interface iface {
        //   type t = Bar;
        //   defn f(z: Bar) -> Bar;
        // }
        // module mdl {
        //   type t = Bar;
        //   defn f(z: Bar) -> Bar {
        //     z
        //   }
        // }
        sexp!(
            (Unit #:path #(test)
             (Item #:ident Bar #:kind #:class
              (Variant
               (FieldDef)
               (FieldDef)))
             (Item #:ident foo #:kind #:defn (Generics)
              (FnSig
               (FnDecl
                #((Param x (Ty #:kind #:path #(Bar)))
                  (Param y (Ty #:kind #:path #(Bar))))
                (FnRetTy (Ty #:kind #:path #(Bar)))))
              (Block
               (Stmt #:kind #:semi (Expr #:kind #:path #(x)))
               (Stmt #:kind #:expr (Expr #:kind #:path #(y)))))
             (Item #:ident iface #:kind #:interface
              (Signature
               (Decl #:ident t #:kind #:ty (Ty #:kind #:path #(Bar)))
               (Decl #:ident f #:kind #:defn (Generics)
                (FnSig (FnDecl #((Param z (Ty #:kind #:path #(Bar)))))))))
             (Item #:ident mdl #:kind #:module
              (Structure
               (Binding #:ident t #:kind #:ty (Ty #:kind #:path #(Bar)))
               (Binding #:ident f #:kind #:defn (Generics)
                (FnSig (FnDecl
                        #((Param z (Ty #:kind #:path #(Bar))))
                        (FnRetTy (Ty #:kind #:path #(Bar)))))
                (Block
                 (Stmt #:kind #:expr (Expr #:kind #:path #(z)))))))))
    }

    #[test]
    fn test_ast_to_hil() {
        use crate::ast::to_hil;
        let ast = get_ast();
        let hil = to_hil(&ast);
        assert!(
            sexp_match(
                &hil,
                &sexp!(
                   (Unit #:path #(test)
                    (Item #:def_id 0 #:ident Bar #:kind #:class
                     (Variant #:hil_id #:_ #:def_id 1
                      (FieldDef #:hil_id #:_ #:def_id 2)
                      (FieldDef #:hil_id #:_ #:def_id 3)))
                    (Item #:def_id 4 #:ident foo #:kind #:defn (Generics #:hil_id #:_)
                     (FnSig #:hil_id #:_
                      (FnDecl #:hil_id #:_
                       #((Param #:hil_id #:_1 x
                          (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))
                         (Param #:hil_id #:_2 y
                          (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0)))))
                       (FnRetTy #:hil_id #:_ (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))))
                     (Block #:hil_id #:_
                      (Stmt #:hil_id #:_ #:kind #:semi
                       (Expr #:hil_id #:_ #:kind #:qpath (Path #:kind #:local #:_1)))
                      (Stmt #:hil_id #:_ #:kind #:expr
                       (Expr #:hil_id #:_ #:kind #:qpath (Path #:kind #:local #:_2)))))
                   (Item #:def_id 5 #:ident iface #:kind #:interface
                    (Signature #:hil_id #:_
                     (Decl #:hil_id #:_ #:def_id 6 #:ident t #:kind #:ty
                      (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))
                     (Decl #:hil_id #:_ #:def_id 7 #:ident f #:kind #:defn (Generics #:hil_id #:_)
                      (FnSig #:hil_id #:_
                       (FnDecl #:hil_id #:_
                        #((Param #:hil_id #:_ z
                           (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))))))))
                    (Item #:def_id 8 #:ident mdl #:kind #:module
                     (Structure #:hil_id #:_
                      (Binding #:hil_id #:_ #:def_id 9 #:ident t #:kind #:ty
                       (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))
                      (Binding #:hil_id #:_ #:def_id 10 #:ident f #:kind #:defn (Generics #:hil_id #:_)
                       (FnSig #:hil_id #:_
                        (FnDecl #:hil_id #:_
                         #((Param #:hil_id #:_3 z
                            (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0)))))
                         (FnRetTy #:hil_id #:_ (Ty #:hil_id #:_ #:kind #:qpath (Path #:kind #:def (DefId 0 0))))))
                       (Block #:hil_id #:_
                        (Stmt #:hil_id #:_ #:kind #:expr
                         (Expr #:hil_id #:_ #:kind #:qpath (Path #:kind #:local #:_3))))))))),
                &mut MatchContext::new(),
            )
        );
    }

    #[test]
    fn test_hil_to_bui() {
        use crate::ast::to_hil;
        use crate::hil::to_bui;
        use crate::context::GlobalContext;
        let ast = get_ast();
        let hil = to_hil(&ast);
        let ctx = GlobalContext::new();
        let bui = to_bui(&hil, &ctx).deserialize();
        let items = bui.items();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].ident(), "Bar");
        if let Some(items_0) = items[0].class() {
            assert_eq!(items_0.fields().len(), 0);
        } else {
            assert!(false, "iterm 0 must a class.");
        }
        assert_eq!(items[1].ident(), "foo");
        if let Some(items_1) = items[1].defn() {
            let fn_sig = items_1.fn_sig();
            let decl = fn_sig.decl();
            let inputs = decl.inputs();
            assert_eq!(inputs.len(), 2);
            let ty = inputs[0].adt();
            assert!(ty.is_some());
            let t = ty.unwrap();
            assert_eq!(t.unit(), 0);
            assert_eq!(t.def(), 0);

            let ty = inputs[1].adt();
            assert!(ty.is_some());
            let t = ty.unwrap();
            assert_eq!(t.unit(), 0);
            assert_eq!(t.def(), 0);
        } else {
            assert!(false, "item 1 must be a defn.");
        }
        assert_eq!(items[2].ident(), "iface");
        if let Some(items_2) = items[2].interface() {
            assert_eq!(items_2.types().len(), 1);
            let (ident, ty_decl) = &items_2.types()[0];
            assert_eq!(*ident, "t");
            let transparent = ty_decl.transparent();
            assert!(transparent.is_some());
            let ty = transparent.unwrap();
            let adt = ty.adt();
            assert!(adt.is_some());
            let t = adt.unwrap();
            assert_eq!(t.unit(), 0);
            assert_eq!(t.def(), 0);

            assert_eq!(items_2.defns().len(), 1);
            let (ident, _) = items_2.defns()[0];
            assert_eq!(ident, "f");
        } else {
            assert!(false, "item 2 must be an iface");
        }
        assert_eq!(items[3].ident(), "mdl");
        if let Some(items_3) = items[3].module() {
            assert_eq!(items_3.types().len(), 1);

            assert_eq!(items_3.types().len(), 1);
            let (ident, ty) = &items_3.types()[0];
            assert_eq!(*ident, "t");
            let adt = ty.adt();
            assert!(adt.is_some());
            let t = adt.unwrap();
            assert_eq!(t.unit(), 0);
            assert_eq!(t.def(), 0);

            assert_eq!(items_3.defns().len(), 1);
            let (ident, _) = items_3.defns()[0];
            assert_eq!(ident, "f");
            assert_eq!(items_3.defns().len(), 1);
        } else {
            assert!(false, "item 3 must be a module");
        }
    }

    #[test]
    fn test_hil_to_til() {
        use crate::ast::to_hil;
        use crate::hil::to_til;
        use crate::t::ty_check;
        use crate::context::GlobalContext;
        let ast = get_ast();
        let hil = to_hil(&ast);
        let ctx = GlobalContext::new();
        let tctx = ty_check(&ctx, &hil);
        let til = to_til(&hil, &ctx, &tctx);
    }
}
