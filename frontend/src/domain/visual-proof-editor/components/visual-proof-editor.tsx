import { css } from '@emotion/css';
import React, { useEffect, useState } from 'react';
import { VisualProofEditorSidebar, VisualProofEditorRule, Assumption } from './visual-proof-editor-sidebar';
import { ProofTreeRule, Prop } from 'alice';
import { handleAndIntroRule } from '../proof-rule/proof-rule-handler/handle-and-intro-rule';
import { v4 } from 'uuid';
import { notification } from 'antd';
import { handleImplIntroRule } from '../proof-rule/proof-rule-handler/handle-impl-intro-rule';
import { handleOrIntroFstRule } from '../proof-rule/proof-rule-handler/handle-or-intro-fst-rule';
import { handleOrIntroSndRule } from '../proof-rule/proof-rule-handler/handle-or-intro-snd-rule';
import { DndContext, DragEndEvent, DragStartEvent, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { SelectedProofTreeNode, VisualProofEditorProofTreeView, VisualProofEditorProofTreeViewId } from './visual-proof-editor-proof-tree-view';
import { VisualProofEditorAssumptionView } from './visual-proof-editor-assumptiom-view';
import { handleAndElimFstRule } from '../proof-rule/proof-rule-handler/handle-and-elim-fst-rule';
import { handleAndElimSndRule } from '../proof-rule/proof-rule-handler/handle-and-elim-snd-rule';
import { TrashOverlayId } from './trash-overlay';
import bg from '../../../../style/bg.png';
import { isEqual } from 'lodash';
import { handleTrueIntroRule } from '../proof-rule/proof-rule-handler/handle-true-intro-rule';
import { handleFalsumElimRule } from '../proof-rule/proof-rule-handler/handle-falsum-elimm-rule';

export interface ReasoningContext {
    id: string;
    proofTree: VisualProofEditorProofTree;
    isActive: boolean;
    x: number;
    y: number;
}

export interface VisualProofEditorProofTree {
    id: string,
    premisses: VisualProofEditorProofTree[],
    rule: ProofTreeRule | null,
    conclusion: Prop,
}

interface VisualProofEditorProps {
    prop: Prop;
}

export function VisualProofEditor({ prop }: VisualProofEditorProps) {
    const [reasoningContexts, setReasoningContexts] = useState<ReasoningContext[]>([]);
    const [selectedNode, setSelectedNode] = useState<SelectedProofTreeNode | null>(null);

    const [assumptions, setAssumptions] = useState<Assumption[]>([]);

    const [notificationApi, contextHolder] = notification.useNotification();

    const reset = () => {

        setReasoningContexts([{
            id: v4(),
            proofTree: {
                id: v4(),
                premisses: [],
                rule: null,
                conclusion: prop,
            },
            isActive: false,
            x: 0,
            y: 0,
        }]);

        setSelectedNode(null);
        setAssumptions([]);
    };

    // setup new tree when prop changes
    useEffect(() => {
        reset()
    }, [prop]);

    const handleRuleSelect = (ruleId: string) => {
        if (!selectedNode) {
            notificationApi.error({
                message: 'Select a proof tree node first.',
            })
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

        try {
            const { newProofTree, additionalAssumptions } = rule.handler({ ...selectedTree });
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
        } catch (err) {
            console.error(err);

            let message = 'Unknown error';
            if (err instanceof Error) {
                message = err.message;
            }

            notificationApi.error({
                message,
            });
        }
    };

    const handleNodeSelect = (selection: SelectedProofTreeNode) => {
        setSelectedNode(selection);
    };

    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 8,
            },
        })
    );

    const onDragStart = (e: DragStartEvent) => {
        const contextId = e.active.id;
        const context = reasoningContexts.find((ctx) => ctx.id === contextId);

        if (!context) {
            return;
        }
        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== contextId),
            {
                ...context,
                isActive: true,
            }
        ]);
    };

    const onDragEnd = (e: DragEndEvent) => {
        const dropsOnArea = e.over?.id === VisualProofEditorProofTreeViewId;
        const dropsOnTrash = e.over?.id === TrashOverlayId;

        const contextId = e.active.id;
        const context = reasoningContexts.find((ctx) => ctx.id === contextId);
        const deltaX = e.delta.x;
        const deltaY = e.delta.y;

        if (!context) {
            throw new Error('Unknwon context: ' + contextId);
        }

        if (dropsOnArea) {
            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== contextId),
                {
                    ...context,
                    isActive: false,
                    x: context.x + deltaX,
                    y: context.y + deltaY,
                }
            ]);

            return;
        }

        // TODO delete assumptions
        if (dropsOnTrash) {
            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== contextId),
            ]);

            return;
        }

        // merge proof trees

        // active: thingy that got dropped (this is a reasoning context).
        // over: thingy that active got dropped on.

        const droppedReasoningCtx = reasoningContexts.find((ctx) => ctx.id === e.active.id);
        const droppedOnReasoningCtx = reasoningContexts.find((ctx) => ctx.id === e.over?.data.current?.contextId)
        const droppedOnPremisseId = e.over?.data.current?.nodeId;


        if (droppedReasoningCtx && droppedOnReasoningCtx) {
            const droppedOnPremisse = getTreeNodeById(droppedOnReasoningCtx.proofTree, droppedOnPremisseId);
            const droppedConclusion = droppedReasoningCtx.proofTree.conclusion;

            if (!isEqual(droppedOnPremisse?.conclusion, droppedConclusion)) {
                notificationApi.error({message: 'Proof trees not compatible.'});
            }

            replaceTreeNodeById(droppedOnReasoningCtx.proofTree, droppedOnPremisse!.id, droppedReasoningCtx.proofTree);

            const newReasoningCtx: ReasoningContext = {
                ...droppedOnReasoningCtx,
                id: v4(),
                proofTree: droppedOnReasoningCtx.proofTree,
            };


            setReasoningContexts([
                ...reasoningContexts.filter((ctx) => ctx.id !== droppedOnReasoningCtx.id && ctx.id !== droppedReasoningCtx.id),
                newReasoningCtx,
            ]);

            console.log(droppedOnPremisse);

            return;
        }

        // restore position
        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== contextId),
            {
                ...context,
                isActive: false,
            }
        ]);
    };

    const onAssumptionClick = (assumptiom: Assumption) => {

        if (assumptiom.kind !== 'PropIsTrue') {
            throw Error('TODO!');
        }

        const conclusion = assumptiom.prop;

        setReasoningContexts([
            ...reasoningContexts,
            {
                id: v4(),
                isActive: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [],
                    rule: {kind: 'Ident', value: assumptiom.ident},
                    conclusion,
                }
            }
        ]);
    };

    return (
        <div>
            {contextHolder}
            <div className={cssVisualProofEditorContent}>
                <VisualProofEditorSidebar rules={NaturalDeductionRules} onRuleSelect={handleRuleSelect} />
                <div style={{ border: '2px solid #37485f', width: '100%', display: 'flex', flexDirection: 'column', background: `url(${bg})` }}>
                    <DndContext sensors={sensors} onDragStart={onDragStart} onDragEnd={onDragEnd} onDragCancel={() => console.log('cancel')}>
                        <VisualProofEditorAssumptionView assumptions={assumptions} onAssumptionClick={onAssumptionClick} onResetClick={reset} />
                        <VisualProofEditorProofTreeView
                            contexts={reasoningContexts}
                            handleNodeSelect={handleNodeSelect} />
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

function getProofRule(id: string): VisualProofEditorRule {
    for (const rule of NaturalDeductionRules) {
        if (rule.id === id) {
            return rule;
        }
    }

    throw new Error(`Unknown rule ${id}`);
}

export type NaturalDeductionRule = 'TrueIntro' | 'AndIntro' | 'AndElimFst' | 'AndElimSnd' | 'ImplIntro' | 'ImplElim' | 'OrIntroFst' | 'OrIntroSnd' | 'OrElim' | 'FalsumElim' | 'ForAllIntro' | 'ForAllElim' | 'ExistsIntro' | 'ExistsElim' | 'Hypothesis';

const NaturalDeductionRules: VisualProofEditorRule[] = [
    {
        id: 'TrueIntro',
        name: 'Truth Introduction',
        reasoning: 'BottomUp',
        handler: handleTrueIntroRule,
    },
    {
        id: 'FalsumElim',
        name: 'Falsum Elim',
        reasoning: 'BottomUp',
        handler: handleFalsumElimRule,
    },
    {
        id: 'AndIntro',
        name: 'And Intro',
        reasoning: 'BottomUp',
        handler: handleAndIntroRule,
    },
    {
        id: 'AndElimFst',
        name: 'And Elim Fst',
        reasoning: 'TopDown',
        handler: handleAndElimFstRule,
    },
    {
        id: 'AndElimSnd',
        name: 'And Elim Snd',
        reasoning: 'TopDown',
        handler: handleAndElimSndRule,
    },
    {
        id: 'ImplIntro',
        name: 'Implication Introduction',
        reasoning: 'BottomUp',
        handler: handleImplIntroRule,
    },
    // {
    //     id: 'ImplElim',
    //     name: 'Implication Elimination',
    //     reasoning: 'BottomUp',
    //     handler: handleImplElimRule,
    // },
    {
        id: 'OrIntroFst',
        name: 'Or Introduction Fst',
        reasoning: 'BottomUp',
        handler: handleOrIntroFstRule,
    },
    {
        id: 'OrIntroSnd',
        name: 'Or Introduction Snd',
        reasoning: 'BottomUp',
        handler: handleOrIntroSndRule,
    },
    // {
    //     id: 'OrElim',
    //     name: 'Or Elimination',
    //     reasoning: 'TopDown',
    // },
    // {
    //     id: 'FalsumElim',
    //     name: 'Falsum Elimination',
    //     reasoning: 'BottomUp',
    // },
    // {
    //     id: 'ForAllIntro',
    //     name: 'Universal quantification Introduction',
    //     reasoning: 'BottomUp',
    // },
    // {
    //     id: 'ForAllElim',
    //     name: 'Universal quantification Elimination',
    //     reasoning: 'BottomUp',
    // },
    // {
    //     id: 'ExistsIntro',
    //     name: 'Existential quantification Introduction',
    //     reasoning: 'BottomUp',
    // },
    // {
    //     id: 'ExistsElim',
    //     name: 'Existential quantification Elimination',
    //     reasoning: 'BottomUp',
    // }
];

const cssVisualProofEditorContent = css`
    width: 100%;
    height: 50vh;

    display: flex;
    flex-direction: row;
`;
