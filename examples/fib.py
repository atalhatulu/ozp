def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

import time
start = time.time()
print(fib(35))
print(time.time() - start)
