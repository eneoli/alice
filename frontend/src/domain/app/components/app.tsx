import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { VisualProofEditor, VisualProofEditorProofTree } from '../../visual-proof-editor/components/visual-proof-editor';
import { ConfigProvider, ThemeConfig } from 'antd';
import { Prop, parse_prop, verify } from 'alice';
import { debounce, isEqual } from 'lodash';

export function App() {

    const [proofTerm, setProofTerm] = useState('');
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


    const handleProofTreeChange = (proofTree: VisualProofEditorProofTree) => {
        const code = generateCode(proofTree);
        console.log(code);
        setProofTerm(code);
    };

    const handleVerify = (prop: string) => {
        let isProof = false;
        try {
            verify(prop, proofTerm);
            isProof = true;
        } catch (e) {
            console.error(e);
        }

        console.log(isProof);
    };

    return (
        <ConfigProvider theme={theme}>
            <Header onPropChange={handlePropChange} onVerify={handleVerify} />

            {prop && (
                <>
                    <VisualProofEditor prop={prop} onProofTreeChange={handleProofTreeChange} />

                    <div style={{ marginTop: 20 }}>
                        <CodeEditor height={'30vh'} initialValue={proofTerm} onChange={setProofTerm} />
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

const generateCode: (proofTree: VisualProofEditorProofTree) => string = (proofTree: VisualProofEditorProofTree) => {

    if (proofTree.rule === null) {
        return 'sorry';
    }

    const rule = proofTree.rule;

    switch (rule.kind) {
        case 'TrueIntro': return '()';
        case 'Ident': return rule.value;
        case 'AndIntro': return `(${generateCode(proofTree.premisses[0])}, ${generateCode(proofTree.premisses[1])})`;
        case 'AndElimFst': return `fst (${generateCode(proofTree.premisses[0])})`;
        case 'AndElimSnd': return `snd (${generateCode(proofTree.premisses[0])})`;
        case 'OrIntroFst': return `inl (${generateCode(proofTree.premisses[0])})`;
        case 'OrIntroSnd': return `inr (${generateCode(proofTree.premisses[0])})`;
        case 'OrElim': return `case ${generateCode(proofTree.premisses[0])} of inl ${rule.value[0]} => ${generateCode(proofTree.premisses[1])}, inr ${rule.value[1]} => ${generateCode(proofTree.premisses[2])}`;
        case 'ImplIntro': return `fn ${rule.value} => ${generateCode(proofTree.premisses[0])}`;
        case 'ImplElim': return `(${generateCode(proofTree.premisses[0])}) (${generateCode(proofTree.premisses[1])})`;
        case 'FalsumElim': return `abort (${generateCode(proofTree.premisses[0])})`;
        case 'ForAllIntro': return `fn ${rule.value} => ${generateCode(proofTree.premisses[0])}`;
        case 'ForAllElim': return `(${generateCode(proofTree.premisses[0])}) (${generateCode(proofTree.premisses[1])})`;
        case 'ExistsIntro': return `(${generateCode(proofTree.premisses[0])}, ${generateCode(proofTree.premisses[1])})`;
        case 'ExistsElim': return `let (${rule.value[0]}, ${rule.value[1]}) = ${generateCode(proofTree.premisses[0])} in ${generateCode(proofTree.premisses[1])}`;
    }
};