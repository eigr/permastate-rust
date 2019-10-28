use std::path::Path;
use protoc_rust::Customize;

extern crate protoc_rust;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        //.out_dir(Path::new("src/protos"))
        .compile(
            &[
                "proto/google/proto/empty.proto",
                "proto/google/proto/any.proto",
                "proto/google/proto/descriptor.proto",
                "proto/protocol/cloudstate/entity.proto"/*,
                "proto/protocol/cloudstate/crdt.proto",
                "proto/protocol/cloudstate/event_sourced.proto",
                "proto/protocol/cloudstate/function.proto"*/
                ],
            &["proto"],
        )?;

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["proto/example/shoppingcart/shoppingcart.proto", "proto/example/shoppingcart/persistence/domain.proto"],
        includes: &["proto"],
        customize: Customize {
            ..Default::default()
        },
    }).expect("protoc");

    /*protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/protos",
        input: &["proto/example/shoppingcart/shoppingcart.proto", "proto/example/shoppingcart/persistence/domain.proto"],
        includes: &["proto"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
    }).expect("protoc");*/


    Ok(())
}