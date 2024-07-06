import React from 'react';
import { css } from '@emotion/css';
import { NaturalDeductionRule, VisualProofEditorProofTree } from './visual-proof-editor';
import { partition } from 'lodash';
import { Prop } from 'alice';
import { Button } from 'antd';

export type Assumption = { kind: 'PropIsTrue', prop: Prop, ident: string } | { kind: 'Datatype', datatype: string, ident: string };

export interface ProofRuleHandlerResult {
    newProofTree: VisualProofEditorProofTree,
    additionalAssumptions: Assumption[],
}

type VisualProofEditorRuleHandler = (proofTree: VisualProofEditorProofTree) => ProofRuleHandlerResult;

export interface VisualProofEditorRule {
    id: NaturalDeductionRule;
    name: string;
    reasoning: 'TopDown' | 'BottomUp';
    handler: VisualProofEditorRuleHandler;
}

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