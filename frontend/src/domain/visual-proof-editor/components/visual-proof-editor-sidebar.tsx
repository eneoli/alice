import React from 'react';
import { css } from '@emotion/css';
import { partition } from 'lodash';
import { Button } from 'antd';
import { VisualProofEditorRule } from '../proof-rule';

interface VisualProofEditorSidebarProps {
    rules: VisualProofEditorRule[];
    onRuleSelect: (id: string) => void;
}

export function VisualProofEditorSidebar(props: VisualProofEditorSidebarProps) {

    const { rules, onRuleSelect } = props;

    const [topDownRules, bottomUpRules] = partition(rules, (rule) => rule.reasoning === 'TopDown');

    return (
        <div className={cssVisualProofEditorSidebar}>
            <span>Top Down Reasoning</span>
            <ul className={cssVisualProofEditorRuleList}>
                {
                    topDownRules.map((rule) => (
                        <li key={rule.id}>
                            <Button onClick={() => onRuleSelect(rule.id)}>{rule.name}</Button>
                        </li>
                    ))
                }
            </ul>
            <span>Bottom Up Reasoning</span>
            <ul className={cssVisualProofEditorRuleList}>
                {
                    bottomUpRules.map((rule) => (
                        <li key={rule.id}>
                            <Button onClick={() => onRuleSelect(rule.id)}>{rule.name}</Button>
                        </li>
                    ))
                }
            </ul>
        </div>
    );
}

const cssVisualProofEditorSidebar = css`
    display: flex;
    flex-direction: column;
    border: 2px solid #37485f;
`;

const cssVisualProofEditorRuleList = css`
    padding: 0;
    list-style: none;
`;