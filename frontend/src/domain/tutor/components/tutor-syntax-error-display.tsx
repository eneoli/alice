import React from 'react';
import Convert from 'ansi-to-html';
import { css } from '@emotion/css';

const convert = new Convert();

interface TutorSyntaxErrorDisplayProps {
    errorMessage: string
}

export function TutorSyntaxErrorDisplay(props: TutorSyntaxErrorDisplayProps) {
    const { errorMessage } = props;


    return (
        <>
            <span className={cssHeading}>💥 We have Syntax Errors</span>
            <br />
            <br />
            <div className={cssConsoleContainer}>
                <span className={cssErrorContainer}
                    dangerouslySetInnerHTML={{ __html: convert.toHtml(errorMessage) }} />
            </div>
        </>
    );
}

const cssHeading = css`
    color: white;
    font-size: 1.5em;
`;

const cssConsoleContainer = css`
    background: #1e1f22;
    padding: 10px;
    border-radius: 5px;
    overflow: auto;
`;

const cssErrorContainer = css`
    white-space: pre-wrap;
    font-family: monospace;
`;