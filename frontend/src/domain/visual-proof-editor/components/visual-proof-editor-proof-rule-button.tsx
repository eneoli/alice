import { css } from '@emotion/css';
import { MathJax } from 'better-react-mathjax';
import React from 'react';

interface VisualProofEditorProofRuleButtonProps {
    title: string;
    latex: string;
    onClick: () => void;
}

export function VisualProofEditorProofRuleButton(props: VisualProofEditorProofRuleButtonProps) {
    const { title, latex, onClick } = props;

    return (
        <div className={cssProofRuleButton} onClick={onClick}>
            <div className={cssProofRuleButtonHeader}>
                {title}
            </div>
            <div className={cssProofRuleButtonBody}>
                <MathJax>{latex}</MathJax>
            </div>
        </div>
    );
}

const cssProofRuleButton = css`
    border-radius: 5px;
    border: 3px solid #f9f9f9;
    width: 300px;
    height: 150px;
    display: flex;
    flex-direction: column;
    box-shadow: 4.5px 5px 4px 0px rgba(0,0,0,0.2);
    cursor: pointer;
    box-sizing: border-box;

    :hover {
        border: 3px solid #2781e8;
    }

    :active {
        border: 3px solid #074ca6;
    }
`;

const cssProofRuleButtonHeader = css`
    width: 100%;
    height: 40px;
    background-color: #f8f8f8;
    border-bottom: 2px solid #e8e8e8;
    text-align: center;
    display: flex;
    align-items: center;
    justify-content: center;
`;

const cssProofRuleButtonBody = css`
    background-color: white;
    width: 100%;
    height: 110px;
    display: flex;
    align-items: center;
    justify-content: center;
`;