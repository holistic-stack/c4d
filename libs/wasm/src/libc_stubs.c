#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>

int printf(const char *format, ...) { return 0; }
int fprintf(FILE *stream, const char *format, ...) { return 0; }
int snprintf(char *str, size_t size, const char *format, ...) { return 0; }
int sprintf(char *str, const char *format, ...) { return 0; }
int vfprintf(FILE *stream, const char *format, void *ap) { return 0; }
int vsnprintf(char *str, size_t size, const char *format, void *ap) { return 0; }
