import { Prop } from 'alice';
import { v4 } from 'uuid';
import { VisualProofEditorProofTree } from '../domain/visual-proof-editor/components/visual-proof-editor';

export function createEmptyVisualProofEditorProofTreeFromProp(conclusion: Prop): VisualProofEditorProofTree {
    return {
        id: v4(),
        premisses: [],
        rule: null,
        conclusion: { kind: 'PropIsTrue', value: conclusion },
    }
}