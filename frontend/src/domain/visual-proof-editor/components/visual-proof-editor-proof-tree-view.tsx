import React from 'react';
import { DragOverlay, useDroppable } from '@dnd-kit/core';
import { css } from '@emotion/css';
import { ReasoningContext } from './visual-proof-editor';
import { ReasoningContextNodeSelection, ReasoningContextVisualizer } from './reasoning-context-visualizer';
import { createPortal } from 'react-dom';
import { TrashOverlay } from './trash-overlay';

export const VisualProofEditorProofTreeViewId = 'visual-proof-editor-proof-tree-view';

export interface SelectedProofTreeNode {
    reasoningContextId: string;
    nodeId: string;
    isRoot: boolean;
    isLeaf: boolean;
}

interface VisualProofEditorProofTreeViewProps {
    contexts: ReasoningContext[],
    handleNodeSelect: (selection: SelectedProofTreeNode) => void;
}

export function VisualProofEditorProofTreeView(props: VisualProofEditorProofTreeViewProps) {
    const { contexts, handleNodeSelect } = props;

    const { setNodeRef } = useDroppable({
        id: VisualProofEditorProofTreeViewId,
    });

    const onNodeSelect = (contextId: string, selection: ReasoningContextNodeSelection) => {
        handleNodeSelect({
            reasoningContextId: contextId,
            ...selection,
        });
    };

    return (
        <div className={cssVisualProofEditorProofTreeView} ref={setNodeRef}>
            {
                contexts.filter((ctx) => !ctx.isActive).map((ctx) => (
                    <div key={ctx.id}
                        className={cssReasoningContextVisualizerContainer}
                        style={{ top: ctx.y, left: ctx.x }}>
                        <ReasoningContextVisualizer
                            context={ctx}
                            onNodeSelect={(result) => onNodeSelect(ctx.id, result)} />
                    </div>
                ))
            }

            <div className={cssTrashOverlayContainer}>
                <TrashOverlay />
            </div>

            {createPortal(
                <DragOverlay>
                    {
                        contexts.filter((ctx) => ctx.isActive).map((context) => (
                            <ReasoningContextVisualizer
                                key={context.id}
                                context={context}
                                onNodeSelect={(result) => onNodeSelect(context.id, result)} />
                        ))
                    }
                </DragOverlay>,
                document.body)
            }
        </div>
    );
}

const cssVisualProofEditorProofTreeView = css`
    position: relative;
    width: 100%;
    height: 100%;
`;

const cssReasoningContextVisualizerContainer = css`
    position: absolute;
`;

const cssTrashOverlayContainer = css`
    position: absolute;
    right: 0;
    top: 0;
`;