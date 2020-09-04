load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "qdev",
    srcs = [
        "src/bin/main.rs"
    ],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:serde_json",
        "//src/cargo:regex",
        "//src/cargo:nix",
        "//src/libq",
        "//src/freud:patient"
    ],
    visibility = ["//visibility:public",],
)
