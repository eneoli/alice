import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, ThemeConfig } from 'antd';
import { Prop, parse_prop } from 'alice';
import { debounce, isEqual } from 'lodash';

export function App() {

    const [_proofTerm, setProofTerm] = useState('');
    const [_analyzedProofTerm, setAnalyzedProofTerm] = useState('');
    const [prop, setProp] = useState<Prop | null>(null);

    const handlePropChange = debounce((propString: string) => {
        try {
            const newProp = parse_prop(propString);

            if (!isEqual(prop, newProp)) {
                setProp(newProp);
            }

            setAnalyzedProofTerm('');
        } catch (e) {
            setProp(null);
            console.error(e);
        }
    }, 500);

    return (
        <ConfigProvider theme={theme}>
            <Header onPropChange={handlePropChange} onVerify={(_prop) => { }} />

            {prop && (
                <>
                    <VisualProofEditor prop={prop} />

                    <div style={{ marginTop: 20 }}>
                        <CodeEditor height={'30vh'} onChange={setProofTerm} />
                    </div>
                </>
            )}

            {!prop && (
                <div style={{ textAlign: 'center', color: '#192434' }}>
                    <h1>Alice is ready.</h1>
                    <h2>Please enter a proposition to begin.</h2>
                </div>
            )}
        </ConfigProvider>
    );
}

const theme: ThemeConfig = {
    token: {
        colorPrimary: '#006af5;',
    },
};