#ifndef LIBQ_H
#define LIBQ_H
#include <stddef.h>
#include <unistd.h>

// IO Functions
ssize_t full_write(int fd, const char *buffer, size_t len);

#endif