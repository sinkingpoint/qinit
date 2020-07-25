load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "libq",
    srcs = glob(["src/*.rs", "src/**/*.rs"]),
    deps = [
        "//src/cargo:nix",
        "//src/cargo:ring",
        "//src/cargo:libc",
        "//src/cargo:bitflags",
        "//src/cargo:num_enum"
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