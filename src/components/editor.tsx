import { useEffect, useRef } from 'react'

import { Kitoken } from '../../../kitoken/packages/javascript'

import { EditorContent } from '@/editor/editor'
import type { EditorPluginHandle } from '@/editor/plugins/editor'
import { $getText } from '@/editor/utils'
import { editorContent, tokenContent } from '@/stores/index'

export const Editor = () => {
    const editorRef = useRef<EditorPluginHandle | null>(null)
    const onChangeTimeout = useRef<number>(-1)
    const wasEmpty = useRef(true)
    const wasMounted = useRef(false)
    useEffect(() => {
        if (wasMounted.current) {
            return
        }
        wasMounted.current = true
        const init = async () => {
            const data = await (await fetch('/llama2.kit')).arrayBuffer()
            const tokenizer = new Kitoken(new Uint8Array(data))
            const decoder = new TextDecoder()
            const tokenize = (input: string) => {
                if (input.length === 0) {
                    tokenContent.set([])
                    return
                }
                let output: Uint32Array
                try {
                    output = tokenizer.encode(input, true)
                } catch (error: unknown) {
                    console.error(error)
                    return ['<?>', -1] as [string, number]
                }
                const outputList = [...output].map((token) => {
                    try {
                        return [decoder.decode(tokenizer.decode(new Uint32Array([token]), true)), token] as [string, number]
                    } catch (error: unknown) {
                        console.error(error)
                        return ['<?>', token] as [string, number]
                    }
                })
                tokenContent.set(outputList)
            }
            editorContent.subscribe((content) => {
                tokenize(content)
            })
        }
        init().catch((error: unknown) => {
            console.error(error)
        })
    }, [])
    return (
        <EditorContent
            id="input"
            ref={editorRef}
            css={{
                flex: 1,
                padding: 'x2',
                paddingRight: 'x3',
                userSelect: 'text',
                fontSize: 'x3',
                fontWeight: '340',
                overflowWrap: 'break-word',
                wordWrap: 'break-word',
                wordBreak: 'break-word',
                overflow: 'auto',
                minHeight: '100%',
                outline: 'transparent',
                willChange: 'font-size',
                overscrollBehavior: 'none',
                '&[data-has-content=true]': {
                    fontSize: 'xl',
                },
            }}
            onChange={(event) => {
                event.read(() => {
                    clearTimeout(onChangeTimeout.current)
                    const text = $getText(10)
                    const editor = editorRef.current?.root.current
                    if (editor) {
                        if (text.length > 0) {
                            editor.dataset.hasContent = 'true'
                        } else {
                            delete editor.dataset.hasContent
                        }
                    }
                    if (text.length > 0 || wasEmpty.current) {
                        onChangeTimeout.current = setTimeout(() => {
                            editorRef.current?.context.read(() => {
                                editorContent.set($getText(100_000))
                            })
                        }, 10) as unknown as number
                    } else {
                        editorContent.set(text)
                    }
                    wasEmpty.current = text.length === 0
                })
            }}
        />
    )
}
