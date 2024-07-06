import React, { Fragment, MouseEvent, useState } from 'react';
import { css, cx } from '@emotion/css';
import { ReasoningContext, VisualProofEditorProofTree } from './visual-proof-editor';
import { ProofNode } from '../../proof-tree/components/proof-node';
import { printProp } from '../../../util/print-prop';
import { useDraggable, useDroppable } from '@dnd-kit/core';
import { Prop } from 'alice';
import { printProofRule } from '../../../util/print-proof-rule';

export interface ReasoningContextNodeSelection {
    nodeId: string;
    isLeaf: boolean;
    isRoot: boolean;
}

interface ReasoningContextVisualizerProps {
    context: ReasoningContext;
    onNodeSelect: (nodeSelection: ReasoningContextNodeSelection) => void;
}

export function ReasoningContextVisualizer(props: ReasoningContextVisualizerProps) {
    const { context, onNodeSelect } = props;
    const { proofTree } = context;

    const [activeNode, setActiveNode] = useState<string | null>(null);

    const { attributes, listeners, setNodeRef, transform } = useDraggable({ id: context.id });
    const style = transform ? {
        transform: `translate3d(${transform.x}px, ${transform.y}px, 0)`,
    } : undefined;

    const renderTree = (proofTree: VisualProofEditorProofTree, isRoot: boolean) => {
        const isLeaf = proofTree.premisses.length == 0;
        const isSelectable = isRoot || (isLeaf && proofTree.rule === null);
        const isSelected = isSelectable && proofTree.id === activeNode;

        const onNodeClick = (e: MouseEvent) => {
            if (isSelectable) {
                setActiveNode(proofTree.id);
            }

            onNodeSelect({
                nodeId: proofTree.id,
                isLeaf,
                isRoot,
            });

            e.stopPropagation();
        };

        const conclusion = (
            <Conclusion contextId={context.id}
                nodeId={proofTree.id}
                conclusion={proofTree.conclusion}
                isSelectable={isSelectable}
                isSelected={isSelected}
                isDroppable={isSelectable} />
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
    conclusion: Prop;
    isSelectable: boolean;
    isSelected: boolean;
    isDroppable: boolean;
}

function Conclusion(props: ConclusionProps) {

    const { contextId, nodeId, conclusion, isSelectable, isSelected, isDroppable } = props;

    const { setNodeRef, isOver } = useDroppable({
        id: `${contextId};${nodeId}`,
        data: { contextId, nodeId },
        disabled: !isDroppable,
    });

    return (
        <div className={cx({
            [cssProofTreeConclusionContainer]: true,
            [cssSelectableProofTreeConclusionContainer]: isSelectable,
            [cssSelectedProofTreeConclusionContainer]: isSelected,
            [cssDraggedOverProofTreeConclusionContainer]: isOver,
        })} ref={setNodeRef}>
            {printProp(conclusion)}
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