load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary", "rust_library")

rust_binary(
    name = "qinit",
    srcs = [
        "src/bin/main.rs"
    ],
    deps = [
        "//src/qinit:libqinit",
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/libq"
    ],
    visibility = ["//visibility:public",],
)

rust_library(
    name = "libqinit",
    srcs = [
        "src/lib/lib.rs",
        "src/lib/tasks/mod.rs",
        "src/lib/tasks/registry.rs",
        "src/lib/tasks/identifier.rs",
        "src/lib/tasks/task.rs",
        "src/lib/tasks/serde.rs"
    ],
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