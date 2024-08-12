import { Identifier, ProofTree, ProofTreeConclusion, ProofTreeRule, Prop } from 'alice';
import { v4 } from 'uuid';

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
