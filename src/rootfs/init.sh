#!/bin/sh

echo Hello Userland!

exec /sbin/qgetty -R ttyS0
