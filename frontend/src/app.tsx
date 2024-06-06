import React, { useState } from 'react';
import Editor, { Monaco } from '@monaco-editor/react';
import { infer_type } from 'alice';

export function App() {

    const [type, setType] = useState('');

    const [proofTerm, setProofTerm] = useState('// Write your proof here!\n\n');

    function handleEditorWillMount(monaco: Monaco) {
        // here is the monaco instance
        // do something before editor is mounted
        monaco.languages.typescript.javascriptDefaults.setEagerModelSync(true);


        const keywords = ['fn', 'case', 'of', 'datatype'];

        monaco.languages.register({ id: 'alice' });
        monaco.languages.setMonarchTokensProvider('alice', {
            keywords,
            tokenizer: {
                root: [
                    [/@?[a-zA-Z][\w$]*/, {
                        cases: {
                            '@keywords': 'keyword',
                            '@default': 'variable',
                        }
                    }]
                ]
            }
        });


        monaco.editor.defineTheme('alice-theme', {
            base: 'vs',
            inherit: true,
            colors: {},

            rules: [
                { token: 'keyword', foreground: '#ff6600', fontStyle: 'bold' },
            ]
        });
    }

    const inferType = () => {
        const type = infer_type(proofTerm);
        setType(type);

        console.log(type);
    }

    const handleChange = (value: string | undefined) => {

        value = value?.replaceAll('\\forall', '∀');
        value = value?.replaceAll('\\exists', '∃');
        value = value?.replaceAll('\\top', '⊤');
        value = value?.replaceAll('\\bot', '⊥');
        value = value?.replaceAll('\\implies', '→');
        value = value?.replaceAll('\\and', '∧');
        value = value?.replaceAll('\\or', '∨');
        value = value?.replaceAll('\\not', '¬');

        setProofTerm(value || '');
    }

    return (
        <>
            <button onClick={inferType}>Infer type</button>
            <textarea readOnly={true} value={type}></textarea>
            <Editor height="90vh" beforeMount={handleEditorWillMount} onChange={handleChange} value={proofTerm} defaultLanguage='alice' theme='alice-theme' />
        </>
    );
}