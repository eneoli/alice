import React from 'react';
import { DragOverlay, useDroppable } from '@dnd-kit/core';
import { css } from '@emotion/css';
import { ReasoningContextNodeSelection, ReasoningContextVisualizer } from './reasoning-context-visualizer';
import { createPortal } from 'react-dom';
import { TrashOverlay } from './trash-overlay';
import { VisualProofEditorReasoningContext } from '../lib/visual-proof-editor-reasoning-context';

export const VisualProofEditorProofTreeViewId = 'visual-proof-editor-proof-tree-view';

export interface SelectedProofTreeNode {
    reasoningContextId: string;
    nodeId: string;
    isRoot: boolean;
    isLeaf: boolean;
}

export interface RuleDeleteClickHandlerParams {
    contextId: string;
    nodeId: string;
}

interface VisualProofEditorProofTreeViewProps {
    contexts: VisualProofEditorReasoningContext[],
    onNodeSelect: (selection: SelectedProofTreeNode) => void;
    onRuleDeleteClick: (params: RuleDeleteClickHandlerParams) => void;
    onCanvasClick: () => void;
}

export function VisualProofEditorProofTreeView(props: VisualProofEditorProofTreeViewProps) {
    const { contexts, onNodeSelect, onRuleDeleteClick, onCanvasClick } = props;

    const { setNodeRef } = useDroppable({
        id: VisualProofEditorProofTreeViewId,
    });

    const handleNodeSelect = (contextId: string, selection: ReasoningContextNodeSelection) => {
        onNodeSelect({
            reasoningContextId: contextId,
            ...selection,
        });
    };

    const handleRuleDeleteClick = (contextId: string, nodeId: string) => {
        onRuleDeleteClick({
            contextId,
            nodeId,
        });
    };

    return (
        <div className={cssVisualProofEditorProofTreeView}
            ref={setNodeRef}
            onClick={onCanvasClick}>
            {
                contexts.filter((ctx) => !ctx.isDragging).map((ctx) => (
                    <div key={ctx.id}
                        className={cssReasoningContextVisualizerContainer}
                        style={{ left: ctx.x, top: ctx.y }}>
                        <ReasoningContextVisualizer
                            context={ctx}
                            onNodeSelect={(result) => handleNodeSelect(ctx.id, result)}
                            onRuleDeleteClick={(nodeId) => handleRuleDeleteClick(ctx.id, nodeId)}
                        />
                    </div>
                ))
            }

            <div className={cssTrashOverlayContainer}>
                <TrashOverlay />
            </div>

            {createPortal(
                <DragOverlay>
                    {
                        contexts.filter((ctx) => ctx.isDragging).map((ctx) => (
                            <ReasoningContextVisualizer
                                key={ctx.id}
                                context={ctx}
                                onNodeSelect={(selection) => handleNodeSelect(ctx.id, selection)}
                                onRuleDeleteClick={(nodeId) => handleRuleDeleteClick(ctx.id, nodeId)}
                            />
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
    flex: 1;
    overflow: hidden;
`;

const cssReasoningContextVisualizerContainer = css`
    position: absolute;
`;

const cssTrashOverlayContainer = css`
    position: absolute;
    right: 10px;
    top: 10px;
`;