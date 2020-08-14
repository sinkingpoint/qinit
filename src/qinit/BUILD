load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary", "rust_library", "rust_test")

rust_binary(
    name = "qinit",
    srcs = [
        "src/bin/main.rs"
    ],
    deps = [
        ":accountant",
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/libq"
    ],
    visibility = ["//visibility:public",],
)

rust_library(
    name = "accountant",
    srcs = glob(["src/lib/*.rs", "src/lib/**/*.rs", "src/lib/**/**/*.rs"]),
    deps = [
        "//src/cargo:serde",
        "//src/cargo:toml",
        "//src/cargo:nix",
        "//src/libq"
    ],
    proc_macro_deps = [
        "//src/cargo:serde_derive",
    ],
    visibility = ["//visibility:public",],
)

rust_test (
    name = "test_qinit_serde_common",
    srcs = [
        "src/tests/tasks/serde/test_identifier.rs"
    ],
    deps = [
        ":accountant",
        "//src/cargo:toml",
        "//src/cargo:serde"
    ],
    proc_macro_deps = [
        "//src/cargo:serde_derive"
    ]
)
