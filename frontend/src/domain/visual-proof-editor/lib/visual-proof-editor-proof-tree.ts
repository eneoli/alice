import { Identifier, instantiate_free_parameter, ProofTree, ProofTreeConclusion, ProofTreeRule, Prop } from 'alice';
import { v4 } from 'uuid';
import { AssumptionContext } from '../proof-rule';
import { flatten } from 'lodash';

export interface VisualProofEditorProofTree {
    id: string,
    premisses: VisualProofEditorProofTree[],
    rule: ProofTreeRule | null,
    conclusion: ProofTreeConclusion,
}

export function visualProofEditorProofTreeIntoAliceProofTree(proofTree: VisualProofEditorProofTree): ProofTree {
    return {
        premisses: proofTree.premisses.map(visualProofEditorProofTreeIntoAliceProofTree),
        rule: proofTree.rule ?? { kind: 'Sorry' },
        conclusion: proofTree.conclusion,
    }
}

interface AliceProofTreeIntoVisualProofEditorProofTreeResult {
    proofTree: VisualProofEditorProofTree;
    assumptions: AssumptionContext[];
}

export function aliceProofTreeIntoVisualProofEditorProofTree(reasoningContextId: string, proofTree: ProofTree): AliceProofTreeIntoVisualProofEditorProofTreeResult {

    const premisseResults = proofTree.premisses.map(aliceProofTreeIntoVisualProofEditorProofTree.bind(null, reasoningContextId));
    const premisseTrees = premisseResults.map((result) => result.proofTree);
    const premisseAssumptions = flatten(premisseResults.map((result) => result.assumptions));

    const nodeId = v4();

    const visualProofTree: VisualProofEditorProofTree = {
        id: nodeId,
        premisses: premisseTrees,
        rule: proofTree.rule.kind === 'Sorry' ? null : proofTree.rule,
        conclusion: proofTree.conclusion,
    };

    const assumptions: AssumptionContext[] = [];

    if (proofTree.rule.kind === 'ImplIntro') {

        if (proofTree.conclusion.kind !== 'PropIsTrue') {
            throw new Error('Expected conclusion to be a proposition');
        }

        if (proofTree.conclusion.value.kind !== 'Impl') {
            throw new Error('Expected conclusion to be an implication');
        }

        const prop = proofTree.conclusion.value.value[0];

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'PropIsTrue',
                ident: proofTree.rule.value,
                prop,
            }
        });
    }

    if (proofTree.rule.kind === 'ForAllIntro') {

        if (proofTree.conclusion.kind !== 'PropIsTrue') {
            throw new Error('Expected conclusion to be a proposition.');
        }

        if (proofTree.conclusion.value.kind !== 'ForAll') {
            throw new Error('Expected conclsuion to be universal quantification.');
        }

        const datatype = proofTree.conclusion.value.value.object_type_ident;

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'Datatype',
                ident: proofTree.rule.value,
                datatype,
            }
        });
    }

    if (proofTree.rule.kind === 'OrElim') {

        if (proofTree.conclusion.kind !== 'PropIsTrue') {
            throw new Error('Expected conclusion to be a proposition.');
        }

        if (proofTree.conclusion.value.kind !== 'Or') {
            throw new Error('Expected conclusion to be a disjunction.');
        }

        const [fst, snd] = proofTree.conclusion.value.value;

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'PropIsTrue',
                ident: proofTree.rule.value[0],
                prop: fst,
            }
        });

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'PropIsTrue',
                ident: proofTree.rule.value[1],
                prop: snd,
            },
        });
    }

    if (proofTree.rule.kind === 'ExistsElim') {

        if (proofTree.premisses[0].conclusion.kind !== 'PropIsTrue') {
            throw new Error('Expected first premisse to be a proposition.');
        }

        if (proofTree.premisses[0].conclusion.value.kind !== 'Exists') {
            throw new Error('Expected first premisse to be an existential quantification.');
        }

        const datatype = proofTree.premisses[0].conclusion.value.value.object_type_ident;

        const binding_ident = proofTree.premisses[0].conclusion.value.value.object_ident;

        const uninstantiated_body = proofTree.premisses[0].conclusion.value.value.body;

        const instantiated_body = instantiate_free_parameter(uninstantiated_body, binding_ident, proofTree.rule.value[0]);

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'Datatype',
                ident: proofTree.rule.value[0],
                datatype,
            }
        });

        assumptions.push({
            owningReasoningCtxId: reasoningContextId,
            owningNodeId: nodeId,
            assumption: {
                kind: 'PropIsTrue',
                ident: proofTree.rule.value[1],
                prop: instantiated_body,
            }
        });
    }

    return {
        proofTree: visualProofTree,
        assumptions: [...assumptions, ...premisseAssumptions],
    };
}

export function createEmptyVisualProofEditorProofTreeFromConclusion(conclusion: ProofTreeConclusion) {
    switch (conclusion.kind) {
        case 'PropIsTrue': return createEmptyVisualProofEditorProofTreeFromProp(conclusion.value);
        case 'TypeJudgement': return createEmptyVisualProofEditorProofTreeFromTypeJudgment(conclusion.value[0], conclusion.value[1]);
        default: throw new Error('Cannot handle this kind of conclusion');
    }
}

export function createEmptyVisualProofEditorProofTreeFromProp(conclusion: Prop): VisualProofEditorProofTree {
    return {
        id: v4(),
        premisses: [],
        rule: null,
        conclusion: { kind: 'PropIsTrue', value: conclusion },
    }
}

export function createEmptyVisualProofEditorProofTreeFromTypeJudgment(objectIdent: Identifier, typeIdent: string): VisualProofEditorProofTree {
    return {
        id: v4(),
        premisses: [],
        rule: null,
        conclusion: { kind: 'TypeJudgement', value: [objectIdent, typeIdent] },
    }
}

export function getTreeNodeById(root: VisualProofEditorProofTree, id: string): VisualProofEditorProofTree | null {
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

export function replaceTreeNodeById(root: VisualProofEditorProofTree, id: string, replacement: VisualProofEditorProofTree): boolean {
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
