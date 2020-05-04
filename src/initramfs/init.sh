#!/bin/sh

export PATH=/bin

mkdir /proc /sys /dev /tmp

echo "Mounting system devices"
mount -t devtmpfs devtmpfs /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs tmpfs /tmp

echo "Parsing Commandline Flags"
cat /proc/cmdline | read cmdline

for word in $cmdline; do
    case $word in
        root=*) echo "root" $word ;;
        *) echo $word ;;
    esac
done

sh