import React, { Fragment, MouseEvent } from 'react';
import { css, cx } from '@emotion/css';
import { ProofNode } from '../../proof-tree/components/proof-node';
import { printProp, printTypeJudgment } from '../../../util/print-prop';
import { useDraggable, useDroppable } from '@dnd-kit/core';
import { ProofTreeConclusion } from 'alice';
import { printProofRule } from '../../../util/print-proof-rule';
import { VisualProofEditorReasoningContext } from '../lib/visual-proof-editor-reasoning-context';
import { VisualProofEditorProofTree } from '../lib/visual-proof-editor-proof-tree';

export interface ReasoningContextNodeSelection {
    nodeId: string;
    isLeaf: boolean;
    isRoot: boolean;
}

interface ReasoningContextVisualizerProps {
    context: VisualProofEditorReasoningContext;
    onNodeSelect: (nodeSelection: ReasoningContextNodeSelection) => void;
}

export function ReasoningContextVisualizer(props: ReasoningContextVisualizerProps) {
    const { context, onNodeSelect } = props;
    const { proofTree } = context;

    const { attributes, listeners, setNodeRef, transform } = useDraggable({ id: context.id });
    const style = transform ? {
        transform: `translate3d(${transform.x}px, ${transform.y}px, 0)`,
    } : undefined;

    const renderTree = (proofTree: VisualProofEditorProofTree, isRoot: boolean) => {
        const isLeaf = proofTree.premisses.length == 0;
        const isSelectable = isRoot || (isLeaf && proofTree.rule === null);
        const isSelected = isSelectable && proofTree.id === context.selectedNodeId;

        const onNodeClick = (e: MouseEvent) => {
            if (isSelectable) {
                onNodeSelect({
                    nodeId: proofTree.id,
                    isLeaf,
                    isRoot,
                });
            }

            e.stopPropagation();
        };

        const conclusion = (
            <Conclusion contextId={context.id}
                nodeId={proofTree.id}
                conclusion={proofTree.conclusion}
                isSelectable={isSelectable}
                isSelected={isSelected}
                isDroppable={isSelectable && isLeaf}
                isRoot={isRoot} />
        );

        return (
            <div onClick={onNodeClick}>
                <ProofNode rule={proofTree.rule ? printProofRule(proofTree.rule) : null} content={conclusion}>
                    {
                        proofTree.premisses.map((child: VisualProofEditorProofTree, i: number) => (
                            <Fragment key={i}>
                                {renderTree(child, false)}
                            </Fragment>
                        ))
                    }
                </ProofNode>
            </div>
        );
    };

    return (
        <div ref={setNodeRef} className={cssProofTreeContainer} style={style} {...listeners} {...attributes}>
            {renderTree(proofTree, true)}
        </div>
    );
}

const cssProofTreeContainer = css`
    cursor: grab;
`;

interface ConclusionProps {
    contextId: string;
    nodeId: string;
    conclusion: ProofTreeConclusion,
    isSelectable: boolean;
    isSelected: boolean;
    isDroppable: boolean;
    isRoot: boolean;
}

function Conclusion(props: ConclusionProps) {

    const { contextId, nodeId, conclusion, isSelectable, isSelected, isDroppable, isRoot } = props;

    const { setNodeRef, isOver } = useDroppable({
        id: `${contextId};${nodeId}`,
        data: { contextId, nodeId, isRoot },
        disabled: !isDroppable,
    });

    return (
        <div className={cx({
            [cssProofTreeConclusionContainer]: true,
            [cssSelectableProofTreeConclusionContainer]: isSelectable,
            [cssSelectedProofTreeConclusionContainer]: isSelected,
            [cssDraggedOverProofTreeConclusionContainer]: isOver,
        })} ref={setNodeRef}>
            {conclusion.kind === 'PropIsTrue' && (
                printProp(conclusion.value)
            )}

            {conclusion.kind === 'TypeJudgement' && (
                printTypeJudgment(conclusion.value)
            )}
        </div>
    );
}

const cssProofTreeConclusionContainer = css`
    cursor: grab;
    user-selection: none;
    border: 3px solid;
    border-color: transparent;
    border-spacing: 2px;
    box-sizing: border-box;
    user-select: none;
    * {
        user-select: none; 
    }
`;

const cssSelectableProofTreeConclusionContainer = css`
    cursor: pointer;
`;

const cssSelectedProofTreeConclusionContainer = css`
    border-color: green;
`;

const cssDraggedOverProofTreeConclusionContainer = css`
    background-color: #ACE1AF;
`;