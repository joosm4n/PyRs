
import dis

def all():
    def add():
        a = 1 + 2
        print(a)

    def do(fnptr):
        fnptr()

    do(add)

print(dis.dis(all))
