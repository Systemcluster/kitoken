from time import time

from kitoken import Kitoken

print(Kitoken)

encoder = Kitoken.from_file("../../tests/models/sentencepiece/llama2.model")
print(encoder)

en = encoder.encode("hello world!", True)
print(en)
de = encoder.decode(en)
print(de.decode("utf-8"))

assert de.decode("utf-8") == "hello world!"

text = open("../../benches/data/wagahai.txt", "rb").read().decode("utf-8")
now = time()
for i in range(100):
    _ = encoder.encode(text, True)
print(f"100 iterations in {(time() - now) * 1000:.3f}ms")

definition = encoder.definition()
print(definition["meta"])
encoder.set_definition(definition)

conf = encoder.config()
print(conf)
encoder.set_config(conf)
mult = encoder.encode_all(["hello world!", "hello world!"], True)
demu = encoder.decode_all(mult)
assert demu[0].decode("utf-8") == "hello world!"
assert demu[1].decode("utf-8") == "hello world!"

t = encoder.encode("Kitoken. Tokenize Everything!", True)
print(t)
print(encoder.decode(t).decode("utf-8"))
print(encoder.decode_all([[x] for x in t]))

encoder.to_bytes()
print("OK")
