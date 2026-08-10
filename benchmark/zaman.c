#include <stdio.h>
int main() {
    long long i = 0;
    long long sinir = 2000000;
    long long toplam = 0;
    while(i < sinir) {
        toplam = toplam + i;
        i = i + 1;
    }
    printf("Bitti\n");
    return 0;
}
