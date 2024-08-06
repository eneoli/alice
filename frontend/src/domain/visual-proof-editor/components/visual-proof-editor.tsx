import { css } from '@emotion/css';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { VisualProofEditorSidebar } from './visual-proof-editor-sidebar';
import { Prop } from 'alice';
import { notification } from 'antd';
import { DndContext, DragCancelEvent, DragEndEvent, DragStartEvent, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { SelectedProofTreeNode, VisualProofEditorProofTreeView, VisualProofEditorProofTreeViewId } from './visual-proof-editor-proof-tree-view';
import { VisualProofEditorAssumptionView } from './visual-proof-editor-assumptiom-view';
import { TrashOverlayId } from './trash-overlay';
import bg from '../../../../style/bg.png';
import { isEqual } from 'lodash';
import { visualProofEditorCollisionDetection } from '../visual-proof-editor-collision-detection';
import { createIdentifierGenerator } from '../proof-rule/proof-rule-handler/create-identifier-generator';
import { AssumptionContext, NaturalDeductionRules, VisualProofEditorRuleHandlerParams, createProofTreeConclusionFromAssumption, getProofRule } from '../proof-rule';
import { getTreeNodeById, replaceTreeNodeById, VisualProofEditorProofTree } from '../lib/visual-proof-editor-proof-tree';
import { createEmptyVisualProofEditorReasoningContextFromConclusion, createEmptyVisualProofEditorReasoningContextFromProp, VisualProofEditorReasoningContext } from '../lib/visual-proof-editor-reasoning-context';
import { useReasoningContexts } from '../hooks/use-reasoning-contexts';
import { useKeyPressed, useKeyUpEvent } from '../../../lib/hooks/use-key-event';
import { createNumberGenerator } from '../proof-rule/proof-rule-handler/create-number-generator';

interface VisualProofEditorProps {
    prop: Prop;
    onProofTreeChange: (tree: VisualProofEditorProofTree) => void;
}

export function VisualProofEditor({ prop, onProofTreeChange }: VisualProofEditorProps) {

    const isControlKeyActive = useKeyPressed('Control');

    const [notificationApi, contextHolder] = notification.useNotification();

    const {
        reasoningContexts,
        addReasoningContext,
        getReasoningContext,
        removeReasoningContext,
        updateReasoningContext,
        updateReasoningContexts,
    } = useReasoningContexts();
    const [primaryReasoningCtxId, setPrimaryReasoningCtxId] = useState<string | null>(null);
    const [assumptions, setAssumptions] = useState<AssumptionContext[]>([]);
    const { current: generateIdentifier } = useRef(createIdentifierGenerator());
    const { current: generateUniqueNumber } = useRef(createNumberGenerator());

    useKeyUpEvent(() => {
        updateReasoningContexts([
            ...reasoningContexts.map((ctx) => {
                ctx.selectedNodeId = null;
                ctx.isDragging = false;

                return { ...ctx };
            })
        ]);
    }, ['Escape']);

    const resetProofEditor = useCallback(() => {
        const mainCtx = createEmptyVisualProofEditorReasoningContextFromProp(prop);

        updateReasoningContexts([mainCtx]);
        setPrimaryReasoningCtxId(mainCtx.id);
        setAssumptions([]);

        generateIdentifier.reset();
    }, [prop]);

    useEffect(resetProofEditor, [prop]);

    const onDragStart = useCallback((e: DragStartEvent) => {
        const contextId = '' + e.active.id;
        const context = getReasoningContext(contextId);
        if (!context) {
            return;
        }

        updateReasoningContext(contextId, { ...context, isDragging: true });
    }, [getReasoningContext, updateReasoningContext]);

    const onDragEnd = (e: DragEndEvent) => {
        // active: thingy that got dropped (this is a reasoning context).
        // over: thingy that active got dropped on.

        const droppedContextId = '' + e.active.id;
        const droppedContext = getReasoningContext(droppedContextId);

        const dropsOnArea = e.over?.id === VisualProofEditorProofTreeViewId;
        if (dropsOnArea) {
            // update x and y position
            updateReasoningContext(droppedContextId, {
                ...droppedContext,
                isDragging: false,
                x: droppedContext.x + e.delta.x,
                y: droppedContext.y + e.delta.y,
            })

            return;
        }

        const restorePositions = () => {
            updateReasoningContext(droppedContextId, {
                ...droppedContext,
                isDragging: false,
            });
        }

        //  FIXME: Shall I delete trees with assumptions?
        const dropsOnTrash = e.over?.id === TrashOverlayId;
        if (dropsOnTrash) {

            if (droppedContextId === primaryReasoningCtxId) {
                notificationApi.error({ message: 'You cannot delete your main proof tree.' });
                restorePositions();
                return;
            }

            removeReasoningContext(droppedContextId);
            setAssumptions(assumptions.filter((ctx) => ctx.owningReasoningCtxId !== droppedContextId));

            return;
        }

        // merge proof trees

        const droppedOnContextId = e.over?.data.current?.contextId;

        if (!droppedOnContextId) {
            restorePositions();
            return;
        }

        const droppedOnContext = getReasoningContext(droppedOnContextId);

        const droppedOnPremisseId = e.over?.data.current?.nodeId;
        const droppedOnPremisse = getTreeNodeById(droppedOnContext.proofTree, droppedOnPremisseId);

        const droppedOnIsLeaf = droppedOnPremisse?.premisses.length === 0;
        const droppedOnIsRoot = e.over?.data.current?.isRoot;
        const droppedIsPrimary = droppedContextId === primaryReasoningCtxId;

        if (droppedOnIsRoot || !droppedOnIsLeaf || droppedIsPrimary) {
            // do not allow drop on root or itermediate nodes as we don't know how to merge then.
            // also don't allow to merge with primary if primary is dropped.
            restorePositions();
            return;
        }

        const droppedConclusion = droppedContext.proofTree.conclusion;

        // check if proof trees are compatible
        if (!isEqual(droppedOnPremisse?.conclusion, droppedConclusion)) {
            notificationApi.error({ message: 'Proof trees not compatible.' });
            restorePositions();
            return;
        }

        // FIXME: shall I check if all assumptions can be used?

        // Merge

        replaceTreeNodeById(droppedOnContext.proofTree, droppedOnPremisse.id, droppedContext.proofTree);

        updateReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== droppedContext.id && ctx.id !== droppedOnContext.id),
            {
                ...droppedOnContext,
                proofTree: droppedOnContext.proofTree,
            }
        ]);

        const droppedOnIsPrimary = droppedOnContext.id === primaryReasoningCtxId;
        if (droppedOnIsPrimary) {
            onProofTreeChange(droppedOnContext.proofTree);
        }
    };

    const onDragCancel = useCallback((e: DragCancelEvent) => {
        const droppedContextId = '' + e.active.id;

        updateReasoningContext(droppedContextId, {
            ...getReasoningContext(droppedContextId),
            isDragging: false,
        });
    }, [reasoningContexts, getReasoningContext, updateReasoningContext]);

    const handleNodeSelect = useCallback((newSelectedNode: SelectedProofTreeNode) => {
        const changedContext = getReasoningContext(newSelectedNode.reasoningContextId);

        if (isControlKeyActive) {
            updateReasoningContext(changedContext.id, {
                ...changedContext,
                selectedNodeId: newSelectedNode.nodeId
            });
        } else {
            updateReasoningContexts([
                ...reasoningContexts.map((ctx) => {
                    if (ctx.id == newSelectedNode.reasoningContextId) {
                        ctx.selectedNodeId = newSelectedNode.nodeId;
                    } else {
                        ctx.selectedNodeId = null;
                    }
                    return ctx;
                }),
            ]);
        }
    }, [getReasoningContext, updateReasoningContext, updateReasoningContexts, reasoningContexts, isControlKeyActive]);

    const handleCanvasClick = useCallback(() => {
        updateReasoningContexts([
            ...reasoningContexts.map((ctx) => {
                ctx.selectedNodeId = null;

                return ctx;
            })
        ]);
    }, [updateReasoningContexts, reasoningContexts]);

    const onAssumptionClick = useCallback((assumptionCtx: AssumptionContext) => {
        const conclusion = createProofTreeConclusionFromAssumption(assumptionCtx.assumption);
        const newReasoningContext = createEmptyVisualProofEditorReasoningContextFromConclusion(conclusion);
        newReasoningContext.proofTree.rule = { kind: 'Ident', value: assumptionCtx.assumption.ident.name };

        addReasoningContext(newReasoningContext);
    }, [addReasoningContext]);

    const handleRuleSelect = async (ruleId: string) => {
        const selectedNodes: SelectedProofTreeNode[] = getSelectedNodesFromReasoningContexts(reasoningContexts);

        updateReasoningContexts([
            ...reasoningContexts.map((ctx) => {
                ctx.selectedNodeId = null;

                return ctx;
            })
        ]);

        if (selectedNodes.length == 0) {
            notificationApi.error({ message: 'Select a proof tree node first.' })
            return;
        }

        const mapSelectedNodesToProofRuleParams = (node: SelectedProofTreeNode) => {
            const { reasoningContextId, nodeId, isRoot, isLeaf } = node;
            const context = getReasoningContext(reasoningContextId);
            const { proofTree: parentProofTree } = context;
            const selectedTree = getTreeNodeById(parentProofTree, nodeId);

            if (!selectedTree) {
                throw new Error(`Unknown node. parent id:${reasoningContextId}, node id: ${nodeId}`);
            }

            return {
                reasoningContextId: context.id,
                proofTree: { ...selectedTree },
                isRoot,
                isLeaf,
            };
        };

        const handlerParams: VisualProofEditorRuleHandlerParams = {
            generateIdentifier,
            generateUniqueNumber,
            selectedProofTreeNodes: selectedNodes.map(mapSelectedNodesToProofRuleParams),
            assumptions,
            error: (message: string) => notificationApi.error({ message }),
        };

        const rule = getProofRule(ruleId);
        const hasPrimarySelected = !!selectedNodes.find((ctx) => ctx.reasoningContextId === primaryReasoningCtxId);
        if (hasPrimarySelected && rule.handler.willReasonDownwards(handlerParams)) {
            notificationApi.error({ message: 'Cannot destruct conclusion as that\'s what you want to show' });
            return;
        }

        const ruleHandlerResult = await rule.handler.handleRule(handlerParams);

        if (!ruleHandlerResult) {
            return;
        }

        const {
            removedReasoingContextIds,
            newReasoningContexts,
            proofTreeChanges,
            additionalAssumptions
        } = ruleHandlerResult;

        setAssumptions([
            ...assumptions,
            ...additionalAssumptions,
        ]);

        // remove reasoning contexts + handle new
        updateReasoningContexts([
            ...reasoningContexts.filter((ctx) => !removedReasoingContextIds.includes(ctx.id)),
            ...newReasoningContexts,
        ]);

        // handle proof tree changes
        for (const proofTreeChange of proofTreeChanges) {
            const { reasoningContextId, nodeId, newProofTree } = proofTreeChange;
            const context = getReasoningContext(reasoningContextId);

            replaceTreeNodeById(context.proofTree, nodeId, newProofTree);

            // yes, this is a hack to enforce rerendering.
            updateReasoningContext(context.id, { ...context, proofTree: context.proofTree });

            const isPrimary = reasoningContextId === primaryReasoningCtxId;
            if (isPrimary) {
                onProofTreeChange(context.proofTree);
            }
        }
    };

    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 2,
            },
        })
    );

    return (
        <div>
            {contextHolder}
            <div className={cssVisualProofEditorContent}>
                <VisualProofEditorSidebar
                    rules={NaturalDeductionRules}
                    onRuleSelect={handleRuleSelect}
                />
                <div className={cssVisualProofEditorProofTreeViewContainer}>
                    <DndContext
                        collisionDetection={visualProofEditorCollisionDetection}
                        sensors={sensors}
                        onDragStart={onDragStart}
                        onDragEnd={onDragEnd}
                        onDragCancel={onDragCancel}>
                        <VisualProofEditorAssumptionView
                            assumptionContexts={assumptions}
                            onAssumptionClick={onAssumptionClick}
                            onResetClick={resetProofEditor}
                        />
                        <div className={cssDivider} />
                        <VisualProofEditorProofTreeView
                            contexts={reasoningContexts}
                            onNodeSelect={handleNodeSelect}
                            onCanvasClick={handleCanvasClick}
                        />
                    </DndContext>
                </div>
            </div>
        </div>
    )
}

function getSelectedNodesFromReasoningContexts(reasoningContexts: VisualProofEditorReasoningContext[]): SelectedProofTreeNode[] {
    return reasoningContexts
        .filter((ctx) => !!ctx.selectedNodeId)
        .map((ctx) => ({
            reasoningContextId: ctx.id,
            nodeId: ctx.selectedNodeId!,
            isLeaf: getTreeNodeById(ctx.proofTree, ctx.selectedNodeId!)?.rule === null,
            isRoot: ctx.proofTree.id === ctx.selectedNodeId,
        }));
}

const cssVisualProofEditorContent = css`
    width: 100%;
    height: 60vh;

    display: flex;
    flex-direction: row;
`;

const cssVisualProofEditorProofTreeViewContainer = css`
    border: 2px solid #37485f;
    width: 100%;
    display: flex;
    flex-direction: column;
    background: url(${bg});
`;

const cssDivider = css`
    border-bottom: 1px solid #233348;
    width: '99%';
    align-self: center;
`;