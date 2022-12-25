fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/bui.capnp")
        .run().expect("schema compiler command");
}
