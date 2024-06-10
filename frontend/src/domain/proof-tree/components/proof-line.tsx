import { css } from '@emotion/css';
import React, { useEffect, useRef } from 'react';
import Katex from 'katex';

interface ProofLineProps {
    label: string;
    onSizeChange: (size: number) => void;
}

export function ProofLine({ label, onSizeChange }: ProofLineProps) {

    const labelRef = useRef<HTMLDivElement>(null);

    useEffect(() => {

        if (!labelRef.current) {
            return;
        }

        Katex.render(label, labelRef.current, {
            throwOnError: false,

        });

        setImmediate(() => onSizeChange(labelRef.current?.offsetWidth || 0));

    }, [labelRef.current]);

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
`;

const cssLine = css`
    flex: 1;
    height: 2px;
    background-color: #5b6e97;
    border-color: #5b6e97;
    color: #5b6e97;
    box-sizing: border-box;
`;