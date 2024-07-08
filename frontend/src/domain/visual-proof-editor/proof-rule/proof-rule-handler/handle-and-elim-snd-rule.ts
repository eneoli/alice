import { v4 } from 'uuid';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export async function handleAndElimSndRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind != 'And') {
        throw new Error('Conclusion is not a conjunction');
    }

    const [_fst, snd] = conclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: v4(),
            premisses: [proofTree],
            rule: { kind: 'AndElimSnd' },
            conclusion: snd,
        }
    };
}