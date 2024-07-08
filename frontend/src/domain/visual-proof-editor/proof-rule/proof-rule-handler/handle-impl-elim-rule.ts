import { v4 } from 'uuid';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export async function handleImplElimRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'Impl') {
        throw new Error('Conclusion is not an implication');
    }

    const [fst, snd] = conclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: v4(),
            premisses: [{ ...proofTree }, {
                id: v4(),
                premisses: [],
                rule: null,
                conclusion: fst,
            }],
            rule: { kind: 'ImplElim' },
            conclusion: snd,
        }
    };
}