#include <string.h>
#include <malloc.h>
#include <unistd.h>
#include "libq.h"

int main(int argc, char **argv) {
    ((void)argc);
    char **start = argv;
    size_t bufferlen = 0;
    while (*++argv) {
        bufferlen += strlen(*argv) + 1;
    }
    argv = start;
    char *buffer = malloc(bufferlen);
    char *bufferEnd = buffer;
    while(*++argv) {
        bufferEnd = stpcpy(bufferEnd, *argv);
        if(*(argv + 1)) {
            *bufferEnd++ = ' ';
        }
    }
    *bufferEnd++ = '\n';
    full_write(STDOUT_FILENO, buffer, bufferEnd-buffer);
    free(buffer);
    return 0;
}