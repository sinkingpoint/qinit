#include "libq.h"
#include <errno.h>

ssize_t full_write(int fd, const char *buffer, size_t len) {
    size_t count = 0;
    while(count < len) {
        ssize_t n = write(fd, buffer, len);
        if(n <= 0) { // We got an error, or we're done
            if(count == 0) {
                return errno; // If we didn't write anything, return -1
            }
            else {
                return count;
            }
        }

        count += n;
        buffer += n;
    }

    return count;
}