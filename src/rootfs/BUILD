load("//src/initramfs:initramfs.bzl", "initramfs")

initramfs(
    name = "rootfs",
    bins = [
        "//src/coreutils:cat",
        "//src/coreutils:echo",
        "//src/coreutils:mknod",
        "//src/coreutils:mkdir",
        "//src/coreutils:mount",
        "//src/coreutils:ls",
        "//src/utils:clear",
        "//src/utils:shutdown",
        "//src/freud:freudian",
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
    extra_files = {
        "tasks/shutdown.service": "/etc/qinit/tasks/shutdown.service"
    },
    init_script = "init.sh",
    out = "rootfs.ext4"
)