import React, { useState } from 'react';
import { Header } from './header';
import { CodeEditor } from '../../code-editor/components/code-editor';
import { verify } from 'alice';

export function App() {

    const [proofTerm, setProofTerm] = useState('');

    return (
        <>
            <Header onVerify={(prop) => alert(verify(prop, proofTerm))}/>
            <CodeEditor onChange={setProofTerm}/>
        </>
    );
}