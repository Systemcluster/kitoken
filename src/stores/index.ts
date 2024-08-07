import { atom } from 'nanostores'

export const defaultContent = 'Kitoken. Tokenize Everything!'
export const editorContent: ReturnType<typeof atom<string>> = atom(defaultContent)

export const listContent: ReturnType<typeof atom<[string, number][]>> = atom([])
