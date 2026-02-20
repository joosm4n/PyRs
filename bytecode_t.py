
import dis

class vec2:
    y = 0
    def y():
        print("\ny\n")

class bec:
    x = vec2

def a():
    v = bec
    v.x = 1

def b():
    v = bec
    v.x.y = 1

print(dis.dis(a))
print(dis.dis(b))
