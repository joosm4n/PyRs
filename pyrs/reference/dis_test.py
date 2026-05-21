
import dis

class vec2:
    x = 0
    y = 0

def do():
    x = [1, 2, 3]
    v = vec2
    v.x = 2
    print(v.x)

print(do)
print(type(do))

print(dis.code_info(do), "\nBytecode:\n")
print(dis.dis(do))