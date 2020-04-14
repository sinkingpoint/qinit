.PHONY: initramfs
initramfs:
	bazel build //src/initramfs:initramfs

.PHONY: run
run:
	qemu-system-x86_64 -kernel bin/vmlinux -initrd bazel-bin/src/initramfs/initramfs.cpio.gz