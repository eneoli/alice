import { css, cx } from '@emotion/css';
import React, { ReactNode, useState } from 'react';
import { ProofLine } from './proof-line';

interface ProofNodeProps {
    label?: string;
    content: ReactNode
    children?: ReactNode
}

export function ProofNode({ children, content, label }: ProofNodeProps) {

    const [padding, setPadding] = useState(0);

    return (
        <div>
            {children && (
                <>
                    <div style={{paddingRight: padding + 5}} className={cssProofNode}>{children}</div>
                    <ProofLine label={label || ''} onSizeChange={(size) => {
                        setPadding(size);
                    }} />
                </>
            )}
            <div style={{paddingRight: padding}} className={cx(cssProofNode, cssProofNodeContent)}>
                {content}
            </div>
        </div>
    );
}

const cssProofNode = css`
    width: 100%;
    display: inline-flex;
    flex-direction: row;
    gap: 25px;
    justify-content: space-evenly;
    align-items: flex-end;
    box-sizing: border-box;
`;

const cssProofNodeContent = css`
    min-width: max-content;
    box-sizing: border-box;
`;