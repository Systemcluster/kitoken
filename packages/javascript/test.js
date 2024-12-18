import assert from "node:assert";
import fs from "node:fs";

import { Kitoken } from "kitoken/node";

console.debug(Kitoken);

const model = fs.readFileSync("../../tests/models/sentencepiece/llama2.model");
const encoder = new Kitoken(model);
console.debug(encoder);

const en = encoder.encode("hello world!", true);
console.debug(en);
const de = encoder.decode(en);
console.debug(new TextDecoder().decode(de));

assert.equal(new TextDecoder().decode(de), "hello world!");

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
const demu = encoder.decode_all(mult);
assert.equal(new TextDecoder().decode(demu[0]), "hello world!");
assert.equal(new TextDecoder().decode(demu[1]), "hello world!");

const t = encoder.encode("Kitoken. Tokenize Everything!", true);
console.debug(t);
console.debug(new TextDecoder().decode(encoder.decode(t)));
console.debug(encoder.decode_all([...t].map(x => [x])).map(x => new TextDecoder().decode(x)));

encoder.to_bytes()
console.info("OK");
