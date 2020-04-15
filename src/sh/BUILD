load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "sh",
    srcs = ["src/main.rs"],
    deps = ["//src/libq"],
    visibility = ["//visibility:public",],
)