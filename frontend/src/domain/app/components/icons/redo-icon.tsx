import React, { CSSProperties } from 'react';

interface RedoIconProps {
    style?: CSSProperties;
}

export function RedoIcon({ style }: RedoIconProps) {
    return (
        <svg style={style} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6">
            <path strokeLinecap="round" strokeLinejoin="round" d="m15 15 6-6m0 0-6-6m6 6H9a6 6 0 0 0 0 12h3" />
        </svg>
    );
}