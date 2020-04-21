load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "sh",
    srcs = [
        "src/main.rs",
        "src/process.rs",
        "src/shell.rs",
        "src/builtins.rs"
    ],
    deps = [
        "//src/libq",
        "//src/libq/cargo:nix"
    ],
    visibility = ["//visibility:public",],
)
