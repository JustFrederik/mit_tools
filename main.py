import mit_tools

import time

def test():
    start = time.time_ns()
    data = mit_tools.Data(None)
    trans: mit_tools.Translate = data.get_new_translator_instance_py()
    v = trans.translate_py("Hallo Welt", data)
    print(v)
    print((time.time_ns() - start) * 1e-6)

test()

