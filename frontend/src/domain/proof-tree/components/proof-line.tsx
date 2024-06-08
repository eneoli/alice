import { css } from '@emotion/css';
import React, { useEffect, useRef } from 'react';
import Katex from 'katex';

interface ProofLineProps {
    label: string;
}

export function ProofLine({label}: ProofLineProps) {

    const labelRef = useRef(null);

    useEffect(() => {

        if (!labelRef.current) {
            return;
        }

        Katex.render(label, labelRef.current, {
            throwOnError: false,
        });
    });

    return (
        <span className={cssProofLineContainer}>
            <hr className={cssLine} />
            <div>
                <div ref={labelRef} />
            </div>
        </span>
    );
}

const cssProofLineContainer = css`
    display: flex;
    flex-direction: row;
    padding-left: 10px;
    padding-right: 10px;
`;

const cssLine = css`
    flex: 1;
    height: 2px;
    background-color: #5b6e97;
    border-color: #5b6e97;
    color: #5b6e97;
    box-sizing: border-box;
`;