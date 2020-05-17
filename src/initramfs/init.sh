#!/bin/sh

export PATH=/bin

mkdir /proc /sys /dev /tmp /run

echo "Mounting system devices"
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /run

echo "Parsing Commandline Flags"
cat /proc/cmdline | read cmdline

for word in $cmdline; do
    case $word in
        root=*)
            root_dev=${word#root=}
        ;;
        *) echo $word ;;
    esac
done

mkdir /.root
mount $root_dev /.root
mkdir /.root/dev /.root/proc /.root/sys /.root/tmp /.root/run

exec switch_root /.root /init init
