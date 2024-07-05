import React, { Fragment, MouseEvent, useState } from 'react';
import { css, cx } from '@emotion/css';
import { ReasoningContext, VisualProofEditorProofTree } from './visual-proof-editor';
import { ProofNode } from '../../proof-tree/components/proof-node';
import { printProp } from '../../../util/print-prop';
import { useDraggable, useDroppable } from '@dnd-kit/core';
import { Prop } from 'alice';

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
        const isSelectable = isRoot || isLeaf;
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
            <Conclusion id={proofTree.id}
                conclusion={proofTree.conclusion}
                isSelectable={isSelectable}
                isSelected={isSelected}
                isDroppable={isLeaf || isRoot} />
        );

        return (
            <div onClick={onNodeClick}>
                <ProofNode rule={proofTree.rule} content={conclusion}>
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
        <div ref={setNodeRef} style={style} {...listeners} {...attributes}>
            {renderTree(proofTree, true)}
        </div>
    );
}

interface ConclusionProps {
    id: string;
    conclusion: Prop;
    isSelectable: boolean;
    isSelected: boolean;
    isDroppable: boolean;
}

function Conclusion(props: ConclusionProps) {

    const { id, conclusion, isSelectable, isSelected, isDroppable } = props;

    const { setNodeRef, isOver, over, } = useDroppable({ id, disabled: !isDroppable });

    const isOverDifferentElement = isOver && over?.id == id;

    return (
        <div className={cx({
            [cssProofTreeContainer]: true,
            [cssSelectableProofContainer]: isSelectable,
            [cssSelectedProofContainer]: isSelected,
            [cssDraggedOverProofContainer]: isOverDifferentElement,
        })} ref={setNodeRef}>
            {printProp(conclusion)}
        </div>
    );
}

const cssProofTreeContainer = css`
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

const cssSelectableProofContainer = css`
    cursor: pointer;
`;

const cssSelectedProofContainer = css`
    border-color: green;
`;

const cssDraggedOverProofContainer = css`
    background-color: #ACE1AF;
`;