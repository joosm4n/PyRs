
def sum_a(a):
    s = 0
    r = range(0, a + 1, 1)
    for v in r:
        s += v
    return s

def choice(s):
    if s == "loop":
        return True
    else:
        return False

def empty_fn():
    pass


x = sum_a(5)
print(x)

y = choice("loop")
print(y)

z = empty_fn()
print(z)