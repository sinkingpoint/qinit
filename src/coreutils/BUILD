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
        "//src/libq",
        "//src/cargo:nix",
        "//src/cargo:clap"
    ],
)

rust_binary(
    name = "cat",
    srcs = ["cat/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap"
    ],
)

rust_binary(
    name = "mount",
    srcs = ["mount/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/libq",
    ],
)

rust_binary(
    name = "mkdir",
    srcs = ["mkdir/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/libq",
        "//src/cargo:nix",
        "//src/cargo:clap"
    ],
)

rust_binary(
    name = "blkid",
    srcs = ["blkid/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/libq",
    ],
)

rust_binary(
    name = "ls",
    srcs = ["ls/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/libq",
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/cargo:chrono"
    ],
)