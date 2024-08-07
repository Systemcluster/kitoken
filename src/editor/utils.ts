import {
    $createRangeSelection,
    $getNearestNodeFromDOMNode,
    $getRoot,
    $getSelection,
    $isDecoratorNode,
    $isRangeSelection,
    type RangeSelection,
} from 'lexical'

export const $getTextBeforeSelection = (maxLength: number) => {
    const selection = $getSelection()
    if (!$isRangeSelection(selection)) {
        return
    }
    const pre = $createRangeSelection()
    const left = selection.anchor.isBefore(selection.focus) ? selection.anchor : selection.focus
    pre.focus = left
    return pre.getTextContent().slice(-maxLength)
}

export const $getText = (maxLength: number) => {
    const nodes = [...$getRoot().getChildren()]
    nodes.reverse()
    let text = ''
    for (const node of nodes) {
        text = node.getTextContent() + '\n' + text
        if (text.length > maxLength) {
            text = text.slice(-maxLength)
            break
        }
    }
    text = text.slice(0, -1)
    return text
}

export const $isTargetWithinDecorator = (target: HTMLElement): boolean => {
    const node = $getNearestNodeFromDOMNode(target)
    return $isDecoratorNode(node)
}

export const $isSelectionAtEndOfRoot = (selection: RangeSelection) => {
    const focus = selection.focus
    return focus.key === 'root' && focus.offset === $getRoot().getChildrenSize()
}

export const $insertDataTransfer = (dataTransfer: DataTransfer, selection: RangeSelection) => {
    const text = dataTransfer.getData('text/plain')
    if (text) {
        const lines = text.split(/\r?\n/gu)
        for (const line of lines) {
            if (line !== '') {
                selection.insertText(line)
            }
            if (lines.indexOf(line) !== lines.length - 1) {
                selection.insertParagraph()
            }
        }
    }
}
