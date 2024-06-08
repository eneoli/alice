import { css } from '@emotion/css';
import React, { ReactNode } from 'react';
import { ProofLine } from './proof-line';

interface ProofNodeProps {
    label?: string;
    content: ReactNode
    children?: ReactNode
}

export function ProofNode({ children, content, label }: ProofNodeProps) {
    return (
        <div style={{width: '100%'}}>

            {children && (<>
                <div className={cssProofNode}>{children}</div>
                <ProofLine label={label || ''} />
            </>
            )}
            <div className={cssProofNode}>
                {content}
            </div>
        </div>
    );
}

const cssProofNode = css`
    width: 100%;
    display: flex;
    flex: 1;
    flex-direction: row;
    justify-content: space-evenly;
`;