import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';
import { generateIdentifier } from './generate-identifier';
import { createEmptyVisualProofEditorProofTree } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export function handleImplIntroRule(proofTree: VisualProofEditorProofTree): ProofRuleHandlerResult {

    const { conclusion } = proofTree;

    if (proofTree.conclusion.kind != 'Impl') {
        throw new Error('Conclusion is not an implication.');
    }

    const [fst, snd] = proofTree.conclusion.value;

    return {
        newProofTree: {
            id: proofTree.id,
            premisses: [createEmptyVisualProofEditorProofTree(snd)],
            rule: 'ImplIntro',
            conclusion,
        },
        additionalAssumptions: [{
            kind: 'PropIsTrue',
            prop: fst,
            ident: generateIdentifier(),
        }],
    };
}
