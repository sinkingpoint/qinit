load("//src/initramfs:initramfs.bzl", "initramfs")

extra_files = {
    "//src/utils:qgetty": "/sbin/qgetty",
    "//src/utils:login": "/sbin/login",
    "//src/qdevd:qdevd": "/sbin/qdevd",
    "//src/qdev:qdev": "/sbin/qdev",
    "//src/freud:freudian": "/sbin/freudian",
    "vendor/modules/e1000.ko": "/lib/modules/5.7.10-201.fc32.x86_64/kernel/drivers/net/ethernet/intel/e1000/e1000.ko"
}

extra_files.update({local: "/etc/qinit/{}".format(local) for local in glob(["tasks/*"])})
extra_files.update({local: "/{}".format(local) for local in glob(["etc/*"])})

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
        "//src/utils:id",
        "//src/utils:sudo",
        "//src/utils:readelf",
        "//src/utils:modinfo",
        "//src/utils:depmod",
        "//src/freud:freudctl",
        "//src/qinit:qinit",
        "//src/sh",
        "//src/utils:insmod"
    ],
    libs = glob(["vendor/*.so.*"]),
    extra_files = extra_files,
    init_script = "init.sh",
    out = "rootfs.ext4"
)