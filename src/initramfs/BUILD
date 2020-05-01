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
        "//src/coreutils:echo",
        "//src/sh",
    ],
    libs = [
        "vendor/ld-linux-x86-64.so.2",
        "vendor/libc.so.6",
        "vendor/libdl.so.2",
        "vendor/libgcc_s.so.1",
        "vendor/libm.so.6",
        "vendor/libpthread.so.0",
        "vendor/librt.so.1",
        "vendor/libstdc++.so.6",
        "vendor/libtinfo.so.6",
    ],
    init_script = "init.sh",
    out = "initramfs.igz"
)