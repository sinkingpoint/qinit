load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary", "rust_library", "rust_test")

rust_library(
    name = "breuer",
    srcs = glob(["src/lib/server/*.rs"]),
    deps = [
        "//src/libq:libq",
        "//src/cargo:nix"
    ]
)

rust_library(
    name = "patient",
    srcs = glob(["src/lib/client/*.rs"]),
    deps = [
        "//src/libq:libq",
        "//src/cargo:nix",
        ":breuer"
    ],
    visibility = ["//visibility:public",],
)

rust_test(
    name = "breuer_test",
    srcs = glob(["src/lib/test/server/*.rs"]),
    deps = [
        ":breuer"
    ]
)

rust_binary(
    name = "freudian",
    srcs = glob(["src/bin/freudian/*.rs"]),
    deps = [
        "//src/libq:libq",
        "//src/cargo:clap",
        ":breuer"
    ],
    visibility = ["//visibility:public",],
)

rust_binary(
    name = "freudctl",
    srcs = glob(["src/bin/freudctl/*.rs"]),
    deps = [
        "//src/libq:libq",
        "//src/cargo:clap",
        ":patient"
    ],
    visibility = ["//visibility:public",],
)
