fn main() {
    let _ = tonic_build::configure()
        .build_server(false)
        .compile_protos(&["yandex/cloud/ai/stt/v3/stt_service.proto"], &["proto"]);
}
