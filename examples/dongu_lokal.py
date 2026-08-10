import time

def test():
    i = 0
    while i < 10000000:
        i = i + 1

baslangic = time.time()
test()
bitis = time.time()
print("10 Milyon Dongu Python (Lokal) Suresi:")
print(bitis - baslangic)
