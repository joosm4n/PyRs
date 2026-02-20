class vec2:
    x = 0

    def print_vec():
        print("vec")

def do():
    v = vec2
    v.print_vec()

import dis
dis.code_info(do)
dis.dis(do)
