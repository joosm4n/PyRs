
import dis

class vec2:
    x = 0

def a():
    x = vec2
    x.x = 1
    print(x)

print(dis.code_info(a))
print(dis.dis(a))
