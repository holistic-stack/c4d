#include <stdarg.h>
#include <stddef.h>

int printf(const char *format, ...) { return 0; }
int fprintf(void *stream, const char *format, ...) { return 0; }
int snprintf(char *str, size_t size, const char *format, ...) { return 0; }
int sprintf(char *str, const char *format, ...) { return 0; }
int vfprintf(void *stream, const char *format, void *ap) { return 0; }
int vsnprintf(char *str, size_t size, const char *format, void *ap) { return 0; }
