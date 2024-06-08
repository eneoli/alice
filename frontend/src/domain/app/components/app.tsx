import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { verify } from 'alice';
import { ProofTree } from '../../proof-tree/proof-tree';

export function App() {

    const [proofTerm, setProofTerm] = useState('');

    return (
        <>
            <Header onVerify={(prop) => alert(verify(prop, proofTerm))}/>
            <ProofTree/>
            <CodeEditor onChange={setProofTerm}/>
        </>
    );
}