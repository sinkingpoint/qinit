load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "libq",
    srcs = [
        "src/lib.rs",
        "src/strings.rs"
    ],
    deps = [
        "//src/libq/cargo:nix",
    ],
    visibility = ["//visibility:public",],
)

rust_test(
    name = "libq_strings_test",
    srcs = [
        "tests/test_strings.rs",
    ],
    deps = [
        ":libq"
    ]
)