
i = 0
n1 = 0
n2 = 1
n3 = 0 # this is a test comment

print("Fibbonacci: ")
while i < 80:
    n3 = n1 + n2
    print("(", i, ") ", n3)
    n1 = n2
    n2 = n3
    i = i + 1
