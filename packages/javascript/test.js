import assert from "node:assert"
import fs from "node:fs"

import { Kitoken } from 'kitoken/node'

console.log(Kitoken)

const model = fs.readFileSync('../../tests/models/sentencepiece/llama2.model')
const encoder = Kitoken.from_sentencepiece(model)
console.log(encoder)

const en = encoder.encode('hello world!', true)
console.log(en)
const de = encoder.decode(en)
console.log(new TextDecoder().decode(de))

assert.equal(new TextDecoder().decode(de), 'hello world!')
