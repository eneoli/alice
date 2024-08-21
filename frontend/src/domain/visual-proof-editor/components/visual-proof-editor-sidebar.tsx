import React, { Fragment } from 'react';
import { css } from '@emotion/css';
import { SelectedProofTreeNode, VisualProofEditorRule } from '../proof-rule';
import { VisualProofEditorProofRuleButton } from './visual-proof-editor-proof-rule-button';
import { RuleDirection } from '../proof-rule/proof-rule-handler/proof-rule-handler';

interface VisualProofEditorSidebarProps {
    rules: VisualProofEditorRule[];
    selectedNodes: SelectedProofTreeNode[]
    onRuleSelect: (id: string, direction: RuleDirection) => void;
}

export function VisualProofEditorSidebar(props: VisualProofEditorSidebarProps) {

    const { rules, selectedNodes, onRuleSelect } = props;

    return (
        <div className={cssVisualProofEditorSidebar}>
            <span className={cssSidebarTitle}>Inference rules</span>
            {
                rules.map((rule, i) => (
                    <Fragment key={i}>
                        {
                            rule.handler.canReasonUpwards(selectedNodes) && (
                                <VisualProofEditorProofRuleButton
                                    title={rule.name}
                                    direction='Upwards'
                                    latex={rule.handler.getLatexCode()}
                                    onClick={() => onRuleSelect(rule.id, 'Upwards')}
                                />
                            )
                        }
                        {
                            rule.handler.canReasonDownwards(selectedNodes) && (
                                <VisualProofEditorProofRuleButton
                                    title={rule.name}
                                    direction='Downwards'
                                    latex={rule.handler.getLatexCode()}
                                    onClick={() => onRuleSelect(rule.id, 'Downwards')}
                                />
                            )
                        }
                    </Fragment>
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