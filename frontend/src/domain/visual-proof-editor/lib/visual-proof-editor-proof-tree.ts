import { ProofTreeConclusion, ProofTreeRule } from 'alice';

export interface VisualProofEditorProofTree {
    id: string,
    premisses: VisualProofEditorProofTree[],
    rule: ProofTreeRule | null,
    conclusion: ProofTreeConclusion,
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
