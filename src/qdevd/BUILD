load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "qdevd",
    srcs = [
        "src/bin/main.rs"
    ],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/cargo:serde_json",
        "//src/libq",
        "//src/freud:patient"
    ],
    visibility = ["//visibility:public",],
)
