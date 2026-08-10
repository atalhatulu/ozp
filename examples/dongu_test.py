import time

i = 0
baslangic = time.time()

while i < 10000000:
    i = i + 1

bitis = time.time()
print("10 Milyon Dongu Python Suresi:")
print(bitis - baslangic)
