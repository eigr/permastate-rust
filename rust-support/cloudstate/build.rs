
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
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
    Ok(())
}