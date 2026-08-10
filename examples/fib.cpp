#include <iostream>
#include <chrono>

double fib(double n) {
    if (n < 2.0) return n;
    return fib(n - 1.0) + fib(n - 2.0);
}

int main() {
    auto start = std::chrono::high_resolution_clock::now();

    double result = fib(35.0);
    std::cout << result << "\n";

    auto end = std::chrono::high_resolution_clock::now();
    std::chrono::duration<double> elapsed = end - start;
    std::cout << "C++ Suresi: " << elapsed.count() << "\n";
    return 0;
}
