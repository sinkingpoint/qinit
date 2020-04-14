load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "libq",
    srcs = ["src/lib.rs"],
    deps = [
        "//src/libq/cargo:nix",
    ],
    visibility = ["//visibility:public",],
)