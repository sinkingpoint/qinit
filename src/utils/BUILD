load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "switch_root",
    srcs = ["switch_root/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
    ]
)