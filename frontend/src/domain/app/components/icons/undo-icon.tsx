import React, { CSSProperties } from 'react';

interface UndoIconProps {
    style?: CSSProperties;
}

export function UndoIcon({ style }: UndoIconProps) {
    return (
        <svg style={style} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="size-6">
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
        </svg>
    );
}