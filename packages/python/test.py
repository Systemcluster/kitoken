import kitoken

data = open("../../tests/models/nerdstash.model", "rb").read()
encoder = kitoken.Kitoken.from_sentencepiece(data)

text = open("../../benches/data/pride_and_prejudice.txt",
            "r", encoding="utf-8").read()


def test():
    tokens = encoder.encode(text, True)
    decoded = encoder.decode(tokens)
    decoded = decoded.decode('utf-8')
    assert text == decoded


if __name__ == '__main__':
    import timeit
    print(timeit.timeit("test()", globals=locals(), number=500))
