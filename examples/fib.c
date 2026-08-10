#include <stdio.h>
#include <time.h>

double fib(double n) {
    if (n < 2.0) return n;
    return fib(n - 1.0) + fib(n - 2.0);
}

int main() {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    double result = fib(35.0);
    printf("%f\n", result);

    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    printf("C Suresi: %.6f\n", elapsed);
    return 0;
}
