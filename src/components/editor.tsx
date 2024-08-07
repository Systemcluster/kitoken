import { EditorContent } from '../editor/editor'
import { $getText } from '../editor/utils'
import { editorContent } from '../stores/index'

export const Editor = () => {
    return (
        <EditorContent
            id="input"
            css={{
                flex: 1,
                padding: 'x2',
                userSelect: 'text',
                fontSize: 'xl',
                fontWeight: '340',
                overflowWrap: 'break-word',
                wordWrap: 'break-word',
                wordBreak: 'break-word',
                overflow: 'auto',
                minHeight: '100%',
                outline: 'transparent',
                '&:not(:has([data-lexical-text]))': {
                    fontSize: 'x3',
                },
            }}
            onChange={(event) => {
                event.read(() => {
                    editorContent.set($getText(100_000))
                })
            }}
        />
    )
}
