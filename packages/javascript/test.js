import assert from "node:assert";
import fs from "node:fs";

import { Kitoken } from "kitoken/node";

console.debug(Kitoken);

const model = fs.readFileSync("../../tests/models/sentencepiece/llama2.model");
const encoder = Kitoken.from_sentencepiece(model);
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
