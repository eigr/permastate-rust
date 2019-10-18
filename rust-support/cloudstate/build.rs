
fn main() {
    tonic_build::compile_protos("proto/protocol/cloudstate/entity.proto").unwrap();
}