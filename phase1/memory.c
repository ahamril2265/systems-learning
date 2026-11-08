#include <stdio.h>
#include <stdlib.h>

int global_var = 100;
int bss_var;

int main() {

    int local_var = 50;
    int *heap_var = malloc(sizeof(int));

    *heap_var = 200;

    printf("Code: %p\n", (void *)main);
    printf("Data: %p\n", (void *)&global_var);
    printf("BSS: %p\n", (void *)&bss_var);
    printf("Stack: %p\n", (void *)&local_var);
    printf("Heap: %p\n", (void *)heap_var);

    free(heap_var);

    return 0;
}
