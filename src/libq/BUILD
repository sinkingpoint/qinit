load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "libq",
    srcs = [
        "src/lib.rs",
        "src/strings.rs",
        "src/io.rs",
        "src/qnix.rs",
        "src/terminal.rs",
        "src/blkid/mod.rs",
        "src/blkid/btrfs.rs",
        "src/blkid/device.rs",
        "src/blkid/ext.rs",
        "src/blkid/fat.rs",
        "src/blkid/superblock.rs"
    ],
    deps = [
        "//src/cargo:nix",
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