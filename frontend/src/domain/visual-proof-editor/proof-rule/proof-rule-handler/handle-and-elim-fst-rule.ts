import { v4 } from 'uuid';
import { VisualProofEditorProofTree } from '../../components/visual-proof-editor';
import { ProofRuleHandlerResult } from '../../components/visual-proof-editor-sidebar';

export async function handleAndElimFstRule(proofTree: VisualProofEditorProofTree): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not a conjunction');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind != 'And') {
        throw new Error('Conclusion is not a conjunction');
    }

    const [fst, _snd] = propConclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: v4(),
            premisses: [proofTree],
            rule: { kind: 'AndElimFst' },
            conclusion: { kind: 'PropIsTrue', value: fst },
        }
    };
}