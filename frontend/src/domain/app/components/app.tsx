import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, ThemeConfig } from 'antd';

export function App() {

    const [proofTerm, setProofTerm] = useState('');
    const [_analyzedProofTerm, setAnalyzedProofTerm] = useState('');
    const [_prop, setProp] = useState('');

    return (
        <ConfigProvider theme={theme}>
            <Header onPropChange={() => setAnalyzedProofTerm('')} onVerify={(prop) => {
                setProp(prop);
                setAnalyzedProofTerm(proofTerm);
            }} />

            <VisualProofEditor />

            <div style={{ marginTop: 20 }}>
                <CodeEditor onChange={setProofTerm} />
            </div>
        </ConfigProvider>
    );
}

const theme: ThemeConfig = {
    token: {
        colorPrimary: '#006af5;',
    },
};