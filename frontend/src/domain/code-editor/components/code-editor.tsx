import React, { useEffect } from 'react';
import { Editor, Monaco } from '@monaco-editor/react';
import { useState } from 'react';

interface CodeEditorProps {
    height: string;
    initialValue: string;
    onChange: (value: string) => void;
}

function handleEditorWillMount(monaco: Monaco) {
    monaco.languages.typescript.javascriptDefaults.setEagerModelSync(true);

    const keywords = ['fn', 'case', 'of', 'let', 'in', 'atom', 'datatype', 'inl', 'inr', 'fst', 'snd', 'sorry'];

    monaco.languages.register({ id: 'alice' });
    monaco.languages.setMonarchTokensProvider('alice', {
        brackets: [{ token: 'delimiter.parenthesis', open: '(', close: ')' }],
        keywords,
        tokenizer: {
            root: [
                [
                    /@?[a-zA-Z][\w$]*/, {
                        cases: {
                            '@keywords': 'keyword',
                            '@default': 'variable',
                        }
                    }],
                { include: '@whitespace' },],
            whitespace: [
                [/[ \t\r\n]+/, ''],
                [/\/\*/, 'comment', '@comment'],
                [/\/\/.*\\$/, 'comment', '@linecomment'],
                [/\/\/.*$/, 'comment']
            ],
            comment: [
                [/[^/*]+/, 'comment'],
                [/\*\//, 'comment', '@pop'],
                [/[/*]/, 'comment']
            ],
            linecomment: [
                [/.*[^\\]$/, 'comment', '@pop'],
                [/[^]+/, 'comment']
            ],
        },
    });

    monaco.languages.setLanguageConfiguration('alice', {
        comments: {
            lineComment: '//',
            blockComment: ['/*', '*/'],
        },
        brackets: [
            ['(', ')']
        ],
        autoClosingPairs: [
            { open: '(', close: ')' },
        ],
        surroundingPairs: [
            { open: '(', close: ')' },
        ],
    })

    monaco.languages.registerCompletionItemProvider('alice', {
        provideCompletionItems: function (model, position) {
            // Get the text before the cursor
            const word = model.getWordUntilPosition(position);
            const rangeBackslash = {
                startLineNumber: position.lineNumber,
                endLineNumber: position.lineNumber,
                startColumn: word.startColumn - 1,
                endColumn: word.endColumn,
            };

            const range = {
                startLineNumber: position.lineNumber,
                endLineNumber: position.lineNumber,
                startColumn: word.startColumn,
                endColumn: word.endColumn,
            };

            const keywordCompletions = keywords.map((keyword) => ({
                documentation: keyword,
                insertText: keyword,
                kind: monaco.languages.CompletionItemKind.Keyword,
                label: keyword,
                range: range
            }));

            return {
                suggestions: [
                    {
                        documentation: 'ForAll Quantor',
                        insertText: '∀',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\forall',
                        range: rangeBackslash
                    },
                    {
                        documentation: 'Exists Quantor',
                        insertText: '∃',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\exists',
                        range: rangeBackslash
                    },
                    {
                        documentation: 'Logical And',
                        insertText: '∧',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\and',
                        range: rangeBackslash
                    },
                    {
                        documentation: 'Logical Or',
                        insertText: '∨',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\or',
                        range: rangeBackslash
                    },
                    {
                        documentation: 'Logical Implies',
                        insertText: '→',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\implies',
                        range: rangeBackslash
                    },
                    {
                        documentation: 'Logical Not',
                        insertText: '¬',
                        kind: monaco.languages.CompletionItemKind.Snippet,
                        label: '\\not',
                        range: rangeBackslash

                    },
                    ...keywordCompletions
                ],
            };
        },
        triggerCharacters: ['\\']
    });

    monaco.editor.defineTheme('alice-theme', {
        base: 'vs',
        inherit: true,
        colors: {},

        rules: [
            { token: 'keyword', foreground: '#006af5', fontStyle: 'bold' },
        ]
    });
}

export function CodeEditor(props: CodeEditorProps) {

    const { onChange, initialValue, height } = props;

    const [value, setValue] = useState('// Write your proof here!\n\n');

    useEffect(() => {
        setValue(initialValue);
    }, [initialValue]);

    const onValueChange = (value: string | undefined) => {
        setValue(value || '');
        onChange(value || '')
    };

    return (
        <Editor height={height}
            beforeMount={handleEditorWillMount}
            onChange={onValueChange}
            value={value}
            defaultLanguage='alice'
            options={{
                minimap: { enabled: false },
            }}
            theme='alice-theme' />
    );
}