load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "qdevd",
    srcs = [
        "src/bin/main.rs"
    ],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/libq"
    ],
    visibility = ["//visibility:public",],
)
