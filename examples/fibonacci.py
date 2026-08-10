import time

def fib(n):
    if n < 2:
        return n
    return fib(n-1) + fib(n-2)

print("Fibonacci(35) hesaplaniyor...")
start = time.time()
res = fib(35)
print(res)
end = time.time()
diff = end - start
print("Sure (saniye):")
print(diff)
