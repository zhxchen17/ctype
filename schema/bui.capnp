@0xac9bd7a4cba1f5fd;

# BUI == Binary Unit Interface

struct Entry(A, B) {
  key @0 :A;
  value @1 :B;
}

struct File {
  path @0 :Text;
}

struct Import {
  source @0 :File;
  hash @1 :Data; 
}

struct Kind {
  union {
    ty @0 :Void;
    arrow @1 :List(Kind);
  }
}

struct ItemRef {
  unit @0 :UInt16;
  def @1 :UInt32;
}

struct Ty {
  union {
    bool @0 :Void;
    int @1 :Void;
    adt @2 :ItemRef;
    tuple @3 :List(Ty);
  }
}

struct FnDecl {
  inputs @0 :List(Ty);
  output @1 :Ty;
}

struct FnSig {
  decl @0 :FnDecl;
}

struct Defn {
  fnSig @0 :FnSig;
}

struct FieldDef {
  name @0 :Text;
  type @1 :Ty;
}

struct Class {
  fields @0 :List(FieldDef);
}

struct TyDecl {
  union {
    opaque @0 :Void;
    transparent @1 :Ty;
  }
}

struct Interface {
  types @0 :List(Entry(Text, TyDecl));
  defns @1 :List(Entry(Text, Defn));
}

struct Module {
  types @0 :List(Entry(Text, Ty));
  defns @1 :List(Entry(Text, Defn));
  modules @2 :List(Entry(Text, Module));
}

struct Item {
  ident @0 :Text;
  namespace @1 :List(Text);
  def @2: UInt32;
  kind :union {
    interface @3 :Interface;
    class @4 :Class;
    defn @5 :Defn;
    module @6 :Module;
  }
}

struct Unit {
  source @0 :File;
  hash @1 :Data;
  imports @2 :List(Import);
  items @3: List(Item);
}
