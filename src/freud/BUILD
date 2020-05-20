load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_binary")

rust_library(
    name = "libfreudian",
    srcs = [
        "src/lib/freudian/lib.rs",
        "src/lib/freudian/subscriber.rs",
        "src/lib/freudian/topic.rs",
        "src/lib/freudian/api.rs"
    ],
    deps = [
        "//src/libq:libq",
    ],
    visibility = ["//visibility:public",],
)

rust_binary(
    name = "freudian",
    srcs = [
        "src/bin/freudian/main.rs",
        "src/bin/freudian/functions.rs",
    ],
    deps = [
        "//src/freud:libfreudian",
        "//src/libq:libq",
        "//src/cargo:clap",
    ],
    visibility = ["//visibility:public",],
)
