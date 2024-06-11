import { css } from '@emotion/css';
import React, { useEffect, useRef } from 'react';
import Katex from 'katex';

interface ProofLineProps {
    label: string;
}

export function ProofLine({ label }: ProofLineProps) {

    const labelRef = useRef<HTMLDivElement>(null);

    useEffect(() => {

        if (!labelRef.current) {
            return;
        }

        Katex.render(label, labelRef.current, {
            throwOnError: false,

        });
    }, [labelRef.current, label]);

    return (
        <div className={cssLineContainer}>
            <hr className={cssLine} />
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
`;

const cssLine = css`
    width: 100%;
    color: black;
    background-color: black;
    height: 1px;
    margin: 0;
    border: 0;
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