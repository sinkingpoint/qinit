load("//src/initramfs:initramfs.bzl", "initramfs")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "build_initramfs",
    srcs = ["build_initramfs.py"],
    python_version = "PY3",
    srcs_version = "PY3",
    visibility = ["//visibility:public"],
)

initramfs(
    name = "initramfs",
    bins = [
        "//src/coreutils:cat",
        "//src/coreutils:echo",
        "//src/coreutils:mknod",
        "//src/coreutils:mkdir",
        "//src/coreutils:mount",
        "//src/utils:switch_root",
        "//src/sh",
    ],
    libs = glob(["vendor/*.so.*"]),
    init_script = "init.sh",
    out = "initramfs.igz"
)