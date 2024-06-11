import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { verify, parse_proof_term } from 'alice';
import { ProofTermVisualizer } from '../../proof-tree/components/proof-term-visualizer';
import { ProofTreeView } from '../../proof-tree/proof-tree-view';

export function App() {

    const [proofTerm, setProofTerm] = useState('');

    return (
        <>
            <Header onVerify={(prop) => {
                alert(verify(prop, proofTerm));
                console.log(parse_proof_term(proofTerm));
            }} />

            <ProofTreeView>
                <ProofTermVisualizer proofTermString={proofTerm} />
            </ProofTreeView>

            <div style={{ marginTop: 20 }}>
                <CodeEditor onChange={setProofTerm} />
            </div>
        </>
    );
}