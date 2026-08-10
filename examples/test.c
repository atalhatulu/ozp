#include <stdio.h>
#include <time.h>

int main() {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    volatile int i = 0;
    while (i < 10000000) {
        i = i + 1;
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    
    printf("C Suresi: %.9f\n", elapsed);
    return 0;
}
