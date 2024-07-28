import { ProofTreeConclusion, Prop } from 'alice';
import { v4 } from 'uuid';
import { VisualProofEditorProofTree } from '../domain/visual-proof-editor/lib/visual-proof-editor-proof-tree';

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

export function createEmptyVisualProofEditorProofTreeFromTypeJudgment(objectIdent: string, typeIdent: string): VisualProofEditorProofTree {
    return {
        id: v4(),
        premisses: [],
        rule: null,
        conclusion: { kind: 'TypeJudgement', value: [objectIdent, typeIdent] },
    }
}