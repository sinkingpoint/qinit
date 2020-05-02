load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "sh",
    srcs = [
        "src/main.rs",
        "src/shell.rs",
        "src/builtins.rs",
        "src/ast.rs"
    ],
    deps = [
        "//src/libq",
        "//src/cargo:nix"
    ],
    visibility = ["//visibility:public",],
)
