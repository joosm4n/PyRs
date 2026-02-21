import dis

def scope():
    class vec2:
        x = 0

        def print_vec():
            print("vec")

    def do():
        v = vec2
        v.print_vec()

    do()

scope()

dis.code_info(scope)
dis.dis(scope)
