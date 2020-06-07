load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "switch_root",
    srcs = ["switch_root/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
    ]
)

rust_binary(
    name = "clear",
    srcs = ["clear/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/libq",
    ]
)

rust_binary(
    name = "shutdown",
    srcs = ["shutdown/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
    ]
)

rust_binary(
    name = "qgetty",
    srcs = ["qgetty/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/cargo:libc",
        "//src/libq",
    ]
)

rust_binary(
    name = "login",
    srcs = ["login/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/libq",
    ]
)