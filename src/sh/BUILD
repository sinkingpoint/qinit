load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "sh",
    srcs = glob(["src/*.rs"]),
    deps = [
        "//src/libq",
        "//src/cargo:nix",
        "//src/cargo:regex"
    ],
    visibility = ["//visibility:public",],
)
