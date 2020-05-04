#!/bin/sh

export PATH=/bin

mkdir /proc
mount -t proc proc /proc
cat /proc/cmdline | read cmdline

for word in cmdline; do
    case word in
        root=*) 
    esac
done

sh