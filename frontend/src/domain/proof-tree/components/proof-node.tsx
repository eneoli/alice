import { css } from '@emotion/css';
import React, { ReactNode } from 'react';
import { ProofLine } from './proof-line';
import { ProofTreeRule } from 'alice';

interface ProofNodeProps {
    rule: ProofTreeRule | null;
    content: ReactNode
    children?: ReactNode
}

export function ProofNode({ children, content, rule }: ProofNodeProps) {

    return (
        <div className={cssProofNode}>
            <div className={cssPremisse}>
                {children}
            </div>

            {rule && (
                <ProofLine rule={rule} />
            )}

            <div className={cssConclusion}>
                <span className={cssProofNodeContent}>
                    {content}
                </span>
            </div>
        </div>
    );
}

const cssProofNode = css`
    display: flex;
    flex-direction: column;
    font-size: 30px;
    font-family: Computer Modern;
    color: #002D62;
`;

const cssPremisse = css`
    align-self: center;
    display: flex;
    align-items: flex-end;
    gap: 60px;
`;

const cssConclusion = css`
    display: flex;
    flex-direction: row;
    justify-content: center;
`;

const cssProofNodeContent = css`
    min-width: max-content;
    box-sizing: border-box;
    justify-content: center;
    padding-left: 5px;
    padding-right: 5px;
`;