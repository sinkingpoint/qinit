load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary", "rust_library", "rust_test")

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

rust_binary(
    name = "id",
    srcs = ["id/src/main.rs"],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/libq",
    ]
)

rust_binary(
    name = "sudo",
    srcs = [
        "sudo/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/libq",
        ":libsudo"
    ]
)

rust_library(
    name = "libsudo",
    srcs = [
        "sudo/lib/lib.rs",
        "sudo/lib/sudoers.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:clap",
        "//src/libq",
    ]
)

rust_test(
    name = "sudo_sudoers_test",
    srcs = [
        "sudo/tests/test_sudoers.rs",
    ],
    deps = [
        ":libsudo"
    ]
)

rust_binary(
    name = "netc",
    srcs = [
        "netc/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/libq"
    ]
)

rust_binary(
    name = "insmod",
    srcs = [
        "insmod/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/libq"
    ]
)

rust_binary(
    name = "readelf",
    srcs = [
        "readelf/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:nix",
        "//src/libq"
    ]
)

rust_binary(
    name = "modinfo",
    srcs = [
        "modinfo/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:lzma_rs",
        "//src/cargo:nix",
        "//src/libq"
    ]
)

rust_binary(
    name = "depmod",
    srcs = [
        "depmod/src/main.rs"
    ],
    visibility = ["//visibility:public",],
    deps = [
        "//src/cargo:clap",
        "//src/cargo:lzma_rs",
        "//src/cargo:nix",
        "//src/libq"
    ]
)