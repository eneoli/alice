import { Prop } from 'alice';
import { v4 } from 'uuid';

export function createEmptyVisualProofEditorProofTree(conclusion: Prop) {
    return {
        id: v4(),
        premisses: [],
        rule: null,
        conclusion,
    }
}