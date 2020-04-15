.PHONY: initramfs
initramfs:
	bazel build //src/initramfs:initramfs

.PHONY: run
run: initramfs
	qemu-system-x86_64 -kernel bin/vmlinux -initrd bazel-bin/src/initramfs/initramfs.igz -serial stdio -append "console=ttyAMA0 console=ttyS0" --enable-kvm

.PHONY: debug
debug:
	gdb -ex "add-auto-load-safe-path $(pwd)" -ex "file vmlinux" -ex 'set arch i386:x86-64:intel' -ex 'target remote localhost:1234' -ex 'break start_kernel' -ex 'continue' -ex 'disconnect' -ex 'set arch i386:x86-64' -ex 'target remote localhost:1234'
