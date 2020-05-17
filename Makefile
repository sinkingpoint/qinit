.PHONY: initramfs
initramfs:
	mkdir /tmp/initramfs/dev -p
	[[ -e /tmp/initramfs/dev/urandom ]] || sudo mknod -m666 /tmp/initramfs/dev/urandom c 1 9 
	bazel build //src/initramfs:initramfs

.PHONY: rootfs
rootfs:
	bazel build //src/rootfs:rootfs
	chmod u+w bazel-bin/src/rootfs/rootfs.ext4

.PHONY: run
run: initramfs rootfs
	qemu-system-x86_64 -m 1G -kernel bin/vmlinux -initrd bazel-bin/src/initramfs/initramfs.igz -drive format=raw,file=bazel-bin/src/rootfs/rootfs.ext4 -serial stdio -append "console=ttyAMA0 console=ttyS0 root=/dev/sda" --enable-kvm

.PHONY: debug
debug:
	gdb -ex "add-auto-load-safe-path $(pwd)" -ex "file vmlinux" -ex 'set arch i386:x86-64:intel' -ex 'target remote localhost:1234' -ex 'break start_kernel' -ex 'continue' -ex 'disconnect' -ex 'set arch i386:x86-64' -ex 'target remote localhost:1234'
