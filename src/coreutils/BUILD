load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "echo",
    srcs = ["echo/src/main.rs"],
    deps = ["//src/libq"],
    visibility = ["//visibility:public",],
)

rust_binary(
    name = "mknod",
    srcs = ["mknod/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap"
    ],
)