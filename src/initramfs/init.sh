#!/bin/sh

export PATH=/bin

mkdir /proc
mount -t proc proc /proc
cat /proc/cmdline | read cmdline

echo $cmdline

sh