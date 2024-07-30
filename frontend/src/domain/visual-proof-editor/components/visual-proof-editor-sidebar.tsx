import React from 'react';
import { css } from '@emotion/css';
import { VisualProofEditorRule } from '../proof-rule';
import { VisualProofEditorProofRuleButton } from './visual-proof-editor-proof-rule-button';

interface VisualProofEditorSidebarProps {
    rules: VisualProofEditorRule[];
    onRuleSelect: (id: string) => void;
}

export function VisualProofEditorSidebar(props: VisualProofEditorSidebarProps) {

    const { rules, onRuleSelect } = props;

    return (
        <div className={cssVisualProofEditorSidebar}>
            <span className={cssSidebarTitle}>Inference rules</span>
            {
                rules.map((rule, i) => (
                    <VisualProofEditorProofRuleButton
                        key={i}
                        title={rule.name}
                        latex={rule.handler.getLatexCode()}
                        onClick={() => onRuleSelect(rule.id)}
                    />
                ))
            }
        </div>
    );
}

const cssVisualProofEditorSidebar = css`
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    border: 2px solid #37485f;
    overflow-y: auto;
    width: 400px;
    gap: 20px;
`;

const cssSidebarTitle = css`
    margin: 0;
    font-size: 1.5em;
    font-weight: bold;
`;