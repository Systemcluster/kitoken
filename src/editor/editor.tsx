'use client'

import { type InitialConfigType, LexicalComposer } from '@lexical/react/LexicalComposer'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'

import { type EditorState, type LexicalEditor } from 'lexical'
import { forwardRef, memo, useCallback, useMemo, useRef } from 'react'

import { css, cx } from '../../styles/system/css'
import { splitCssProps } from '../../styles/system/jsx'
import type { JsxStyleProps } from '../../styles/system/types'

import { EditorPlugin, type EditorPluginHandle } from './plugins/editor'

type EditorContextProps = JsxStyleProps & {
    id?: string
    ref?: React.MutableRefObject<EditorPluginHandle | null>
    children?: React.ReactNode | React.ReactNode[]
    onChange?: (editorState: EditorState, editor: LexicalEditor, tags: Set<string>) => void
}
export const EditorContent = memo(
    forwardRef<EditorPluginHandle, EditorContextProps>(({ id, children, onChange, ...props }, ref) => {
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
                    ref={ref}
                    className={cx('editor', css(styleProps, cssProps))}
                    spellCheck={false}
                    autoCapitalize="off"
                    autoCorrect="off"
                    autoFocus={false}
                    tabIndex={0}
                    {...restProps}
                />
                <HistoryPlugin delay={200} />
                <OnChangePlugin onChange={onChangeCallback} />
                {children}
            </LexicalComposer>
        )
    })
)
