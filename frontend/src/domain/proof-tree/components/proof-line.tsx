import { css } from '@emotion/css';
import React, { useEffect, useRef, MouseEvent } from 'react';
import Katex from 'katex';
import { ProofTreeRule } from 'alice';
import { printProofRule } from '../../../util/print-proof-rule';
import { Cross } from './cross';

interface ProofLineProps {
    rule: ProofTreeRule;
    onDeleteClick: () => void;
}

export function ProofLine({ rule, onDeleteClick }: ProofLineProps) {

    const labelRef = useRef<HTMLDivElement>(null);

    useEffect(() => {

        if (!labelRef.current) {
            return;
        }

        Katex.render(printProofRule(rule), labelRef.current, {
            throwOnError: false,

        });
    }, [labelRef.current, rule]);

    const handleDeleteClick = (e: MouseEvent) => {
        e.stopPropagation();
        onDeleteClick();
    }

    return (
        <div className={cssLineContainer}>
            <div
                className={cssLine}
                style={{ borderStyle: rule.kind === 'AlphaEquivalent' ? 'dashed' : undefined }}
            />
            <div className={cssLabelContainer}>
                <div className={cssLabel} style={{ display: 'flex' }}>
                    <div ref={labelRef} />
                    <span
                        className='cssProofRuleDeleteButton'
                        style={{ cursor: 'pointer' }}
                        onClick={handleDeleteClick}
                    >
                        <Cross />
                    </span>
                </div>
            </div>
        </div>);
}

const cssLineContainer = css`
    display: flex;
    align-items: center;
    gap: 2px;
    user-select: none;

    .cssProofRuleDeleteButton {
        display: none;
    }

    :hover .cssProofRuleDeleteButton {
        display: block;
    }
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