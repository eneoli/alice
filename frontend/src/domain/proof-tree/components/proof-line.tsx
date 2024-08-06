import { css } from '@emotion/css';
import React, { useEffect, useRef } from 'react';
import Katex from 'katex';
import { ProofTreeRule } from 'alice';
import { printProofRule } from '../../../util/print-proof-rule';

interface ProofLineProps {
    rule: ProofTreeRule;
}

export function ProofLine({ rule }: ProofLineProps) {

    const labelRef = useRef<HTMLDivElement>(null);

    useEffect(() => {

        if (!labelRef.current) {
            return;
        }

        Katex.render(printProofRule(rule), labelRef.current, {
            throwOnError: false,

        });
    }, [labelRef.current, rule]);

    return (
        <div className={cssLineContainer}>
            <div
                className={cssLine}
                style={{ borderStyle: rule.kind === 'AlphaEquivalent' ? 'dashed' : undefined }}
            />
            <div className={cssLabelContainer}>
                <div className={cssLabel}>
                    <div ref={labelRef} />
                </div>
            </div>
        </div>);
}

const cssLineContainer = css`
    display: flex;
    align-items: center;
    gap: 2px;
    user-select: none;
`;

const cssLine = css`
    width: 100%;
    color: #002D62;
    border: 2px solid #002D62;
    margin: 0;
`;

const cssLabelContainer = css`
    width: 0;
    height: 0;
    position: relative;
`;

const cssLabel = css`
    white-space: nowrap;
    paddding-bottom: 25px;
    position: absolute;
    top: -18px;
    font-size: 0.75em;
`;