import { css } from '@emotion/css';
import React, { useEffect, useRef, useState } from 'react';
import { VisualProofEditorSidebar } from './visual-proof-editor-sidebar';
import { ProofTreeConclusion, ProofTreeRule, Prop } from 'alice';
import { v4 } from 'uuid';
import { notification } from 'antd';
import { DndContext, DragEndEvent, DragStartEvent, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { SelectedProofTreeNode, VisualProofEditorProofTreeView, VisualProofEditorProofTreeViewId } from './visual-proof-editor-proof-tree-view';
import { VisualProofEditorAssumptionView } from './visual-proof-editor-assumptiom-view';
import { TrashOverlayId } from './trash-overlay';
import bg from '../../../../style/bg.png';
import { isEqual } from 'lodash';
import { visualProofEditorCollisionDetection } from '../visual-proof-editor-collision-detection';
import { createIdentifierGenerator } from '../proof-rule/proof-rule-handler/create-identifier-generator';
import { AssumptionContext, NaturalDeductionRules, getProofRule } from '../proof-rule';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../util/create-visual-proof-editor-empty-proof-tree';

export interface ReasoningContext {
    id: string;
    proofTree: VisualProofEditorProofTree;
    isDragging: boolean;
    x: number;
    y: number;
}

export interface VisualProofEditorProofTree {
    id: string,
    premisses: VisualProofEditorProofTree[],
    rule: ProofTreeRule | null,
    conclusion: ProofTreeConclusion,
}

interface VisualProofEditorProps {
    prop: Prop;
    onProofTreeChange: (tree: VisualProofEditorProofTree) => void;
}

export function VisualProofEditor({ prop, onProofTreeChange }: VisualProofEditorProps) {
    const [reasoningContexts, setReasoningContexts] = useState<ReasoningContext[]>([]);
    const [primaryReasoningCtxId, setPrimaryReasoningCtxId] = useState<string | null>(null);

    const [selectedNode, setSelectedNode] = useState<SelectedProofTreeNode | null>(null);

    const [assumptions, setAssumptions] = useState<AssumptionContext[]>([]);

    const [notificationApi, contextHolder] = notification.useNotification();

    const { current: generateIdentifier } = useRef(createIdentifierGenerator());

    const reset = () => {
        const ctxId = v4();

        setReasoningContexts([{
            id: ctxId,
            proofTree: createEmptyVisualProofEditorProofTreeFromProp(prop),
            isDragging: false,
            x: 0,
            y: 0,
        }]);

        setPrimaryReasoningCtxId(ctxId);
        setSelectedNode(null);
        setAssumptions([]);
        generateIdentifier.reset();
        console.log('primary', ctxId);
    };

    // setup new tree when prop changes
    useEffect(() => {
        reset()
    }, [prop]);

    const handleRuleSelect = async (ruleId: string) => {
        if (!selectedNode) {
            notificationApi.error({ message: 'Select a proof tree node first.' })
            return;
        }

        const { reasoningContextId, nodeId, isRoot, isLeaf } = selectedNode;
        const context = reasoningContexts.find((ctx) => ctx.id === reasoningContextId);

        if (!context) {
            throw new Error('Unknown reasoning context: ' + reasoningContextId);
        }

        const { proofTree: parentProofTree } = context;
        const selectedTree = getTreeNodeById(parentProofTree, nodeId);

        if (!selectedTree) {
            throw new Error(`Unknown node. parent id:${reasoningContextId}, node id: ${nodeId}`);
        }

        const rule = getProofRule(ruleId);

        if (rule.reasoning === 'BottomUp' && !isLeaf) {
            throw new Error('Cannot reason bottom-up on this node');
        }

        if (rule.reasoning === 'TopDown' && !isRoot) {
            throw new Error('Cannot reason top-down on this node');
        }

        const isPrimary = context.id === primaryReasoningCtxId;
        if (isPrimary && isRoot && rule.reasoning === 'TopDown') {
            throw new Error('Cannot destruct conclusion as that\'s what you want to show');
        }

        const { newProofTree, additionalAssumptions } = await rule.handler({
            proofTree: { ...selectedTree },
            generateIdentifier,
            reasoningContextId: context.id,
        });

        replaceTreeNodeById(parentProofTree, selectedTree.id, newProofTree);

        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== context.id),
            {
                ...context,
                proofTree: parentProofTree,
            }
        ]);

        setAssumptions([
            ...assumptions,
            ...additionalAssumptions,
        ]);

        if (isPrimary) {
            onProofTreeChange(parentProofTree);
        }
    };

    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 5,
            },
        })
    );

    const onDragStart = (e: DragStartEvent) => {
        // mark context as currently dragged.

        const contextId = e.active.id;
        const context = reasoningContexts.find((ctx) => ctx.id === contextId);

        if (!context) {
            return;
        }

        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== contextId),
            {
                ...context,
                isDragging: true,
            }
        ]);
    };

    const onDragEnd = (e: DragEndEvent) => {

        // active: thingy that got dropped (this is a reasoning context).
        // over: thingy that active got dropped on.

        const droppedContextId = e.active.id;
        const droppedContext = reasoningContexts.find((ctx) => ctx.id === droppedContextId);

        if (!droppedContext) {
            throw new Error('Unknwon context: ' + droppedContextId);
        }

        const dropsOnArea = e.over?.id === VisualProofEditorProofTreeViewId;
        if (dropsOnArea) {
            // update x and y position
            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== droppedContextId),
                {
                    ...droppedContext,
                    isDragging: false,
                    x: droppedContext.x + e.delta.x,
                    y: droppedContext.y + e.delta.y,
                }
            ]);

            return;
        }

        // TODO delete trees with assumptions?
        const dropsOnTrash = e.over?.id === TrashOverlayId;
        if (dropsOnTrash) {
            // delete context + assumptions

            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== droppedContextId),
            ]);

            setAssumptions([
                ...assumptions.filter((ctx) => ctx.owningReasoningCtxId !== droppedContextId)
            ]);

            return;
        }

        // merge proof trees

        const restorePositions = () => {
            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== droppedContextId),
                {
                    ...droppedContext,
                    isDragging: false,
                }
            ]);
        }

        const droppedOnContext = reasoningContexts.find((ctx) => ctx.id === e.over?.data.current?.contextId);

        if (!droppedOnContext) {
            restorePositions();
            return;
        }

        const droppedOnPremisseId = e.over?.data.current?.nodeId;
        const droppedOnPremisse = getTreeNodeById(droppedOnContext.proofTree, droppedOnPremisseId);

        const droppedOnIsLeaf = droppedOnPremisse?.premisses.length === 0;
        const droppedOnIsRoot = e.over?.data.current?.isRoot;
        const droppedIsPrimary = droppedContextId === primaryReasoningCtxId;

        if (droppedOnIsRoot || !droppedOnIsLeaf || droppedIsPrimary) {
            // do not allow drop on root or itermediate nodes as we don't know how merge then.
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

        // check if all assumptions can be used TODO


        // Merge

        replaceTreeNodeById(droppedOnContext.proofTree, droppedOnPremisse.id, droppedContext.proofTree);
        const newReasoningCtx: ReasoningContext = {
            ...droppedOnContext,
            proofTree: droppedOnContext.proofTree,
        };

        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== droppedContext.id && ctx.id !== droppedOnContext.id),
            newReasoningCtx,
        ]);

        const droppedOnIsPrimary = droppedOnContext.id === primaryReasoningCtxId;
        if (droppedOnIsPrimary) {
            onProofTreeChange(droppedOnContext.proofTree);
        }
    };

    const onAssumptionClick = (assumptionCtx: AssumptionContext) => {
        const assumption = assumptionCtx.assumption;

        let conclusion: ProofTreeConclusion;
        switch (assumption.kind) {
            case 'PropIsTrue':
                conclusion = { kind: 'PropIsTrue', value: assumption.prop };
                break;
            case 'Datatype':
                conclusion = { kind: 'TypeJudgement', value: [assumption.ident, assumption.datatype] };
                break;
            default: throw new Error('Cannot handle this assumption kind.');
        }

        setReasoningContexts([
            ...reasoningContexts,
            {
                id: v4(),
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [],
                    rule: { kind: 'Ident', value: assumption.ident },
                    conclusion,
                }
            }
        ]);
    };

    const onRuleSelect = (ruleId: string) => {
        handleRuleSelect(ruleId)
            .catch((e) => {
                console.error(e);
                notificationApi.error({ message: e.message ? e.message : 'Unknown error' });
            });
    }

    return (
        <div>
            {contextHolder}
            <div className={cssVisualProofEditorContent}>
                <VisualProofEditorSidebar rules={NaturalDeductionRules} onRuleSelect={onRuleSelect} />
                <div className={cssVisualProofEditorProofTreeViewContainer}>
                    <DndContext collisionDetection={visualProofEditorCollisionDetection}
                        sensors={sensors}
                        onDragStart={onDragStart}
                        onDragEnd={onDragEnd}>
                        <VisualProofEditorAssumptionView
                            assumptionContexts={assumptions}
                            onAssumptionClick={onAssumptionClick}
                            onResetClick={reset} />
                        <VisualProofEditorProofTreeView
                            contexts={reasoningContexts}
                            handleNodeSelect={setSelectedNode} />
                    </DndContext>
                </div>
            </div>
        </div>
    )
}

function getTreeNodeById(root: VisualProofEditorProofTree, id: string): VisualProofEditorProofTree | null {
    if (root.id === id) {
        return root;
    }

    for (let i = 0; i < root.premisses.length; i++) {
        const premisse = root.premisses[i];

        const childResult = getTreeNodeById(premisse, id);

        if (childResult) {
            return childResult;
        }
    }

    return null;
}

function replaceTreeNodeById(root: VisualProofEditorProofTree, id: string, replacement: VisualProofEditorProofTree): boolean {
    if (root.id === id) {
        root.id = replacement.id;
        root.premisses = replacement.premisses;
        root.rule = replacement.rule;
        root.conclusion = replacement.conclusion;

        return true;
    }

    for (let i = 0; i < root.premisses.length; i++) {
        const premisse = root.premisses[i];

        if (replaceTreeNodeById(premisse, id, replacement)) {
            return true;
        }
    }

    return false;
}

const cssVisualProofEditorContent = css`
    width: 100%;
    height: 50vh;

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