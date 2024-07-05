import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { ProofTermVisualizer } from '../../proof-tree/components/proof-term-visualizer';
import { ProofTreeView } from '../../proof-tree/proof-tree-view';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, ThemeConfig } from 'antd';

export function App() {

    const [proofTerm, setProofTerm] = useState('');
    const [analyzedProofTerm, setAnalyzedProofTerm] = useState('');
    const [prop, setProp] = useState('');

    return (
        <>
            <ConfigProvider theme={theme}>
                <Header onPropChange={() => setAnalyzedProofTerm('')} onVerify={(prop) => {
                    setProp(prop);
                    setAnalyzedProofTerm(proofTerm);
                }} />

                <VisualProofEditor />
                <div style={{ marginTop: 20 }}>
                    <CodeEditor onChange={setProofTerm} />
                </div>

                <ProofTreeView>
                    <ProofTermVisualizer proofTermString={analyzedProofTerm} prop={prop} />
                </ProofTreeView>
            </ConfigProvider>
        </>
    );
}

const theme: ThemeConfig = {
    token: {
        colorPrimary: '#006af5;',
    },
};