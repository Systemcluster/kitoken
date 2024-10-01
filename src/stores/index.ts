import { atom } from 'nanostores'

export const defaultContent = 'Kitoken. Tokenize Everything!' // + Array.from({ length: 100 }, (_, i) => `Line ${i + 1}`).join('\n')
export const defaultTokens = [
    ['Kit', 26_240],
    ['oken', 4476],
    ['.', 29_889],
    ['Token', 25_159],
    ['ize', 675],
    ['Everything', 17_296],
    ['!', 29_991],
] as [string, number][]

export const editorContent: ReturnType<typeof atom<string>> = atom(defaultContent)
export const tokenContent: ReturnType<typeof atom<[string, number][]>> = atom([])
