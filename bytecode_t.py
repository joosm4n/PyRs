
import dis

def scope():
    class vec2:
        x = 0

    v = vec2
    v.x = 10
    y = vec2
    print(y.x)

scope()

print(dis.code_info(scope))
print(dis.dis(scope))

