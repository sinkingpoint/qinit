.PHONY: raze
raze:
	cd src/cargo; cargo generate-lockfile; cargo vendor --versioned-dirs --locked; cargo raze

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
	sudo qemu-system-x86_64 -netdev tap,ifname=qemutap0,script=no,downscript=no,id=mynet0 -device e1000,netdev=mynet0,mac=00:12:35:56:78:9a -m 1G -kernel bin/vmlinux -initrd bazel-bin/src/initramfs/initramfs.igz -drive format=raw,file=bazel-bin/src/rootfs/rootfs.ext4 -serial stdio -append "console=ttyAMA0 console=ttyS0 root=/dev/sda 1" --enable-kvm

.PHONY: debug
debug:
	gdb -ex "add-auto-load-safe-path $(pwd)" -ex "file vmlinux" -ex 'set arch i386:x86-64:intel' -ex 'target remote localhost:1234' -ex 'break start_kernel' -ex 'continue' -ex 'disconnect' -ex 'set arch i386:x86-64' -ex 'target remote localhost:1234'
