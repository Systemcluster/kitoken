'use client'

import { $getHtmlContent } from '@lexical/clipboard'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { ContentEditable, type Props } from '@lexical/react/LexicalContentEditable'
import { $moveCharacter, $shouldOverrideDefaultCharacterSelection } from '@lexical/selection'
import { $canShowPlaceholderCurry } from '@lexical/text'
import { mergeRegister } from '@lexical/utils'
import {
    $getAdjacentNode,
    $getSelection,
    $isDecoratorNode,
    $isElementNode,
    $isNodeSelection,
    $isRangeSelection,
    $selectAll,
    COMMAND_PRIORITY_EDITOR,
    CONTROLLED_TEXT_INSERTION_COMMAND,
    COPY_COMMAND,
    CUT_COMMAND,
    type CommandPayloadType,
    DELETE_CHARACTER_COMMAND,
    DELETE_LINE_COMMAND,
    DELETE_WORD_COMMAND,
    DRAGSTART_COMMAND,
    DROP_COMMAND,
    INSERT_LINE_BREAK_COMMAND,
    INSERT_PARAGRAPH_COMMAND,
    KEY_ARROW_DOWN_COMMAND,
    KEY_ARROW_LEFT_COMMAND,
    KEY_ARROW_RIGHT_COMMAND,
    KEY_ARROW_UP_COMMAND,
    KEY_BACKSPACE_COMMAND,
    KEY_DELETE_COMMAND,
    KEY_ENTER_COMMAND,
    type LexicalEditor,
    PASTE_COMMAND,
    REMOVE_TEXT_COMMAND,
    SELECT_ALL_COMMAND,
} from 'lexical'
import { Fragment, forwardRef, useImperativeHandle, useLayoutEffect, useRef, useState } from 'react'

import { HAS_BEFORE_INPUT, IS_APPLE_WEBKIT, IS_IOS, IS_SAFARI } from '../../utils/environment'
import { $insertDataTransfer, $isSelectionAtEndOfRoot, $isTargetWithinDecorator } from '../utils'

const onCopy = (event: CommandPayloadType<typeof COPY_COMMAND>, editor: LexicalEditor): void => {
    editor.update(() => {
        if (!event) return
        const clipboardData = (event as ClipboardEvent).clipboardData
        const selection = $getSelection()
        if (selection !== null && clipboardData != null) {
            event.preventDefault()
            const htmlString = $getHtmlContent(editor)
            if (htmlString) {
                clipboardData.setData('text/html', htmlString)
            }
            clipboardData.setData('text/plain', selection.getTextContent())
        }
    })
}

const onPaste = (event: CommandPayloadType<typeof PASTE_COMMAND>, editor: LexicalEditor): void => {
    event.preventDefault()
    editor.update(
        () => {
            const clipboardData = (event as ClipboardEvent).clipboardData
            const selection = $getSelection()
            if (clipboardData != null && $isRangeSelection(selection)) {
                $insertDataTransfer(clipboardData, selection)
            }
        },
        {
            tag: 'paste',
        }
    )
}

const onCut = (event: CommandPayloadType<typeof CUT_COMMAND>, editor: LexicalEditor): void => {
    onCopy(event, editor)
    editor.update(() => {
        const selection = $getSelection()
        if ($isRangeSelection(selection)) {
            selection.removeText()
        } else if ($isNodeSelection(selection)) {
            selection.getNodes().forEach((node) => {
                node.remove()
            })
        }
    })
}

const registerEditor = (editor: LexicalEditor) => {
    return mergeRegister(
        editor.registerCommand<boolean>(
            DELETE_CHARACTER_COMMAND,
            (isBackward) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.deleteCharacter(isBackward)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<boolean>(
            DELETE_WORD_COMMAND,
            (isBackward) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.deleteWord(isBackward)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<boolean>(
            DELETE_LINE_COMMAND,
            (isBackward) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.deleteLine(isBackward)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<InputEvent | string>(
            CONTROLLED_TEXT_INSERTION_COMMAND,
            (eventOrText) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                if (typeof eventOrText === 'string') {
                    selection.insertText(eventOrText)
                } else {
                    const dataTransfer = eventOrText.dataTransfer
                    if (dataTransfer == null) {
                        const data = eventOrText.data
                        if (data) {
                            selection.insertText(data)
                        }
                    } else {
                        $insertDataTransfer(dataTransfer, selection)
                    }
                }
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            REMOVE_TEXT_COMMAND,
            () => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.removeText()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<boolean>(
            INSERT_LINE_BREAK_COMMAND,
            () => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.insertParagraph()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            INSERT_PARAGRAPH_COMMAND,
            () => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                selection.insertParagraph()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_ARROW_UP_COMMAND,
            (event) => {
                const selection = $getSelection()
                if ($isNodeSelection(selection) && !$isTargetWithinDecorator(event.target as HTMLElement)) {
                    // If selection is on a node, let's try and move selection
                    // back to being a range selection.
                    const nodes = selection.getNodes()
                    if (nodes[0]) {
                        nodes[0].selectPrevious()
                        return true
                    }
                } else if ($isRangeSelection(selection)) {
                    const possibleNode = $getAdjacentNode(selection.focus, true)
                    if ($isDecoratorNode(possibleNode) && !possibleNode.isIsolated() && !possibleNode.isInline()) {
                        possibleNode.selectPrevious()
                        event.preventDefault()
                        return true
                    } else if ($isElementNode(possibleNode) && !possibleNode.isInline() && !possibleNode.canBeEmpty()) {
                        possibleNode.select()
                        event.preventDefault()
                        return true
                    }
                }
                return false
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_ARROW_DOWN_COMMAND,
            (event) => {
                const selection = $getSelection()
                if ($isNodeSelection(selection)) {
                    // If selection is on a node, let's try and move selection
                    // back to being a range selection.
                    const nodes = selection.getNodes()
                    if (nodes[0]) {
                        nodes[0].selectNext(0, 0)
                        return true
                    }
                } else if ($isRangeSelection(selection)) {
                    if ($isSelectionAtEndOfRoot(selection)) {
                        event.preventDefault()
                        return true
                    }
                    const possibleNode = $getAdjacentNode(selection.focus, false)
                    if ($isDecoratorNode(possibleNode) && !possibleNode.isIsolated() && !possibleNode.isInline()) {
                        possibleNode.selectNext()
                        event.preventDefault()
                        return true
                    }
                }
                return false
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_ARROW_LEFT_COMMAND,
            (event) => {
                const selection = $getSelection()
                if ($isNodeSelection(selection)) {
                    // If selection is on a node, let's try and move selection
                    // back to being a range selection.
                    const nodes = selection.getNodes()
                    if (nodes[0]) {
                        event.preventDefault()
                        nodes[0].selectPrevious()
                        return true
                    }
                }
                if (!$isRangeSelection(selection)) {
                    return false
                }
                if ($shouldOverrideDefaultCharacterSelection(selection, true)) {
                    const isHoldingShift = event.shiftKey
                    event.preventDefault()
                    $moveCharacter(selection, isHoldingShift, true)
                    return true
                }
                return false
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_ARROW_RIGHT_COMMAND,
            (event) => {
                const selection = $getSelection()
                if ($isNodeSelection(selection) && !$isTargetWithinDecorator(event.target as HTMLElement)) {
                    // If selection is on a node, let's try and move selection
                    // back to being a range selection.
                    const nodes = selection.getNodes()
                    if (nodes[0]) {
                        event.preventDefault()
                        nodes[0].selectNext(0, 0)
                        return true
                    }
                }
                if (!$isRangeSelection(selection)) {
                    return false
                }
                const isHoldingShift = event.shiftKey
                if ($shouldOverrideDefaultCharacterSelection(selection, false)) {
                    event.preventDefault()
                    $moveCharacter(selection, isHoldingShift, false)
                    return true
                }
                return false
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent | null>(
            KEY_ENTER_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                if (event !== null) {
                    // If we have beforeinput, then we can avoid blocking
                    // the default behavior. This ensures that the iOS can
                    // intercept that we're actually inserting a paragraph,
                    // and autocomplete, autocapitalize etc work as intended.
                    // This can also cause a strange performance issue in
                    // Safari, where there is a noticeable pause due to
                    // preventing the key down of enter.
                    if ((IS_IOS || IS_SAFARI || IS_APPLE_WEBKIT) && HAS_BEFORE_INPUT) {
                        return false
                    }
                    event.preventDefault()
                    if (event.shiftKey) {
                        return editor.dispatchCommand(INSERT_LINE_BREAK_COMMAND, false)
                    }
                }
                return editor.dispatchCommand(INSERT_PARAGRAPH_COMMAND, undefined)
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_BACKSPACE_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                event.preventDefault()
                return selection.isCollapsed()
                    ? editor.dispatchCommand(DELETE_CHARACTER_COMMAND, true)
                    : editor.dispatchCommand(REMOVE_TEXT_COMMAND, null)
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<KeyboardEvent>(
            KEY_DELETE_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                event.preventDefault()
                return selection.isCollapsed()
                    ? editor.dispatchCommand(DELETE_CHARACTER_COMMAND, false)
                    : editor.dispatchCommand(REMOVE_TEXT_COMMAND, null)
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            SELECT_ALL_COMMAND,
            () => {
                $selectAll()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            COPY_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                onCopy(event, editor)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            CUT_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                onCut(event, editor)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand(
            PASTE_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                onPaste(event, editor)
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<DragEvent>(
            DROP_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                // TODO: Make drag and drop work at some point.
                event.preventDefault()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        ),
        editor.registerCommand<DragEvent>(
            DRAGSTART_COMMAND,
            (event) => {
                const selection = $getSelection()
                if (!$isRangeSelection(selection)) {
                    return false
                }
                // TODO: Make drag and drop work at some point.
                event.preventDefault()
                return true
            },
            COMMAND_PRIORITY_EDITOR
        )
    )
}

const canShowPlaceholderFromCurrentEditorState = (editor: LexicalEditor) =>
    editor.getEditorState().read($canShowPlaceholderCurry(editor.isComposing()))

export type EditorPluginProps = Omit<Props, 'placeholder'> & {
    children?: never
    placeholder?: string
}
export interface EditorPluginHandle {
    context: LexicalEditor
}
export const EditorPlugin = forwardRef<EditorPluginHandle, EditorPluginProps>(({ placeholder, ...props }, ref) => {
    const [editor] = useLexicalComposerContext()

    useLayoutEffect(() => {
        return registerEditor(editor)
    }, [editor])
    useLayoutEffect(() => {
        if (props.autoFocus) {
            setTimeout(() => {
                editor.focus()
            }, 0)
        }
    }, [editor])
    useImperativeHandle(ref, () => ({
        context: editor,
    }))

    const [showPlaceholder, setShowPlaceholder] = useState(() => canShowPlaceholderFromCurrentEditorState(editor))
    const showPlaceholderRef = useRef(showPlaceholder)
    useLayoutEffect(() => {
        return mergeRegister(
            editor.registerUpdateListener(() => {
                const canShowPlaceholder = canShowPlaceholderFromCurrentEditorState(editor)
                if (showPlaceholderRef.current === canShowPlaceholder) {
                    return
                }
                showPlaceholderRef.current = canShowPlaceholder
                setShowPlaceholder(canShowPlaceholder)
            }),
            editor.registerEditableListener(() => {
                const canShowPlaceholder = canShowPlaceholderFromCurrentEditorState(editor)
                if (showPlaceholderRef.current === canShowPlaceholder) {
                    return
                }
                showPlaceholderRef.current = canShowPlaceholder
                setShowPlaceholder(canShowPlaceholder)
            })
        )
    }, [editor])

    return (
        <Fragment>
            <ContentEditable
                data-placeholder={showPlaceholder ? placeholder : null}
                {...props}
                placeholder={undefined}
                aria-placeholder={undefined}
            />
        </Fragment>
    )
})
EditorPlugin.displayName = 'EditorPlugin'
