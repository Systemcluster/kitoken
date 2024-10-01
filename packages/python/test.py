from time import time

from kitoken import Kitoken

print(Kitoken)

# model = open(, "rb").read()
encoder = Kitoken.from_file("../../tests/models/sentencepiece/llama2.model")
print(encoder)

en = encoder.encode("hello world!", True)
print(en)
de = encoder.decode(en)
print(de.decode("utf-8"))

assert de.decode("utf-8") == "hello world!"

# this js but in py:
"""
const text = new TextDecoder().decode(
	fs.readFileSync("../../benches/data/wagahai.txt"),
);
const now = Date.now();
for (let i = 0; i < 100; i++) {
	const _ = encoder.encode(text, true);
}
console.info(`100 iterations in ${Date.now() - now}ms`);
"""

text = open("../../benches/data/wagahai.txt", "rb").read().decode("utf-8")
now = time()
for i in range(100):
    _ = encoder.encode(text, True)
print(f"100 iterations in {(time() - now) * 1000:.3f}ms")
