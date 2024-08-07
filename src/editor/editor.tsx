'use client'

import { type InitialConfigType, LexicalComposer } from '@lexical/react/LexicalComposer'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'

import { type EditorState, type LexicalEditor } from 'lexical'
import { useCallback, useMemo, useRef } from 'react'

import { css, cx } from '../../styles/system/css'
import { splitCssProps } from '../../styles/system/jsx'
import type { JsxStyleProps } from '../../styles/system/types'

import { EditorPlugin } from './plugins/editor'

type EditorContextProps = JsxStyleProps & {
    id?: string
    onChange?: (editorState: EditorState, editor: LexicalEditor, tags: Set<string>) => void
}
export const EditorContent = ({ id, onChange, ...props }: EditorContextProps) => {
    const config = useMemo(
        (): InitialConfigType => ({
            namespace: 'gojira',
            editable: true,
            onError: (error) => {
                console.error(error)
            },
            nodes: [],
            theme: {},
        }),
        []
    )

    const onChangeRef = useRef(onChange)
    onChangeRef.current = onChange
    const onChangeCallback = useCallback((editorState: EditorState, editor: LexicalEditor, tags: Set<string>) => {
        onChangeRef.current?.(editorState, editor, tags)
    }, [])

    const [{ css: cssProps, ...styleProps }, restProps] = splitCssProps(props)
    return (
        <LexicalComposer key="editor-context" initialConfig={config}>
            <EditorPlugin
                id={id}
                className={cx('editor', css(styleProps, cssProps))}
                spellCheck={false}
                autoCapitalize="off"
                autoCorrect="off"
                autoFocus={false}
                tabIndex={0}
                {...restProps}
            />
            <HistoryPlugin />
            <OnChangePlugin onChange={onChangeCallback} />
        </LexicalComposer>
    )
}
