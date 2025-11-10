/*
 * SQLite feature probe - outputs version, threading, and compile-time flags
 * as plain text for parsing by the build script
 */

#include <sqlite3.h>
#include <stdio.h>

int main(void) {
    int i;
    const char *opt;

    printf("%d\n", sqlite3_libversion_number());
    printf("%d\n", sqlite3_threadsafe());
    printf("\n");

    /* Compile options, one per line */
    for (i = 0; (opt = sqlite3_compileoption_get(i)) != NULL; i++) {
        printf("%s\n", opt);
    }

    return 0;
}
