import React from 'react';
import { ProofNode } from './components/proof-node';

export function ProofTree() {
    return (
        <>
            <ProofNode content={'A & B'} label='\land I'>
                <ProofNode content={'A'} label='\land E'>
                    <ProofNode content={'A & A'}></ProofNode>
                </ProofNode>
                <ProofNode content={'B'} label='ID'>
                    <ProofNode content={'B'}></ProofNode>
                </ProofNode>
            </ProofNode>
        </>
    );
}
