import assert from "node:assert";
import fs from "node:fs";

import { Kitoken } from "kitoken/node";

console.debug(Kitoken);

const model = fs.readFileSync("../../tests/models/sentencepiece/llama2.model");
const encoder = new Kitoken(model);
console.debug(encoder);

let en = encoder.encode("hello world!", true);
console.debug(en);
let de = encoder.decode(en);
console.debug(new TextDecoder().decode(de));

assert.equal(new TextDecoder().decode(de), "hello world!");

en = encoder.encode("A<s>B", ["control"])
console.debug(en)
assert.deepEqual(en, new Uint32Array([319, 1, 29933]))
de = new TextDecoder().decode(encoder.decode(en, ["control"]))
console.debug(de)
assert.equal(de, "A<s>B")
de = new TextDecoder().decode(encoder.decode(en, []))
console.debug(de)
assert.equal(de, "AB")

const text = new TextDecoder().decode(
    fs.readFileSync("../../benches/data/wagahai.txt"),
);
const now = Date.now();
for (let i = 0; i < 100; i++) {
    const _ = encoder.encode(text, true);
}
console.info(`100 iterations in ${(Date.now() - now).toFixed(3)}ms`);

const definition = encoder.definition();
console.debug(definition.meta);
encoder.set_definition(definition);

const conf = encoder.config();
console.debug(conf);
encoder.set_config(conf);
const mult = encoder.encode_all(["hello world!", "hello world!"], true);
console.debug(mult)
const demu = encoder.decode_all(mult);
console.debug(demu)
assert.equal(new TextDecoder().decode(demu[0]), "hello world!");
assert.equal(new TextDecoder().decode(demu[1]), "hello world!");

const t = encoder.encode("Kitoken. Tokenize Everything!", true);
console.debug(t);
console.debug(new TextDecoder().decode(encoder.decode(t)));
console.debug(encoder.decode_all([...t].map(x => [x])).map(x => new TextDecoder().decode(x)));

encoder.to_bytes()
console.info("OK");
