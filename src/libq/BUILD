load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "libq",
    srcs = [
        "src/lib.rs",
        "src/strings.rs",
        "src/io.rs",
        "src/daemon.rs",
        "src/rand.rs",
        "src/qnix.rs",
        "src/logger.rs",
        "src/terminal.rs",
        "src/blkid/mod.rs",
        "src/blkid/btrfs.rs",
        "src/blkid/device.rs",
        "src/blkid/ext.rs",
        "src/blkid/fat.rs",
        "src/blkid/superblock.rs",
        "src/passwd/mod.rs",
        "src/passwd/users.rs",
        "src/passwd/group.rs",
        "src/passwd/shadow.rs",
        "src/passwd/crypt_sha2.rs",
    ],
    deps = [
        "//src/cargo:nix",
        "//src/cargo:ring"
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

rust_test(
    name = "libq_passwd_test",
    srcs = [
        "tests/test_passwd.rs",
    ],
    deps = [
        ":libq"
    ]
)