#include <iostream>
#include <chrono>

int main() {
    auto start = std::chrono::high_resolution_clock::now();

    volatile int i = 0;
    while (i < 10000000) {
        i = i + 1;
    }

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> elapsed = end - start;
    
    std::cout << "C++ Suresi: " << elapsed.count() << "\n";
    return 0;
}
