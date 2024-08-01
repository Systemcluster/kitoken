from kitoken import Kitoken

print(Kitoken)

model = open('../../tests/models/sentencepiece/llama2.model', 'rb').read()
encoder = Kitoken.from_sentencepiece(model)
print(encoder)

en = encoder.encode('hello world!', True)
print(en)
de = encoder.decode(en)
print(de.decode('utf-8'))

assert de.decode('utf-8') == 'hello world!'
