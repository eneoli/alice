import { v4 } from 'uuid';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';

export async function handleImplElimRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an implication');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'Impl') {
        throw new Error('Conclusion is not an implication');
    }

    const [fst, snd] = propConclusion.value;

    return {
        additionalAssumptions: [],
        newProofTree: {
            id: v4(),
            premisses: [{ ...proofTree }, {
                id: v4(),
                premisses: [],
                rule: null,
                conclusion: { kind: 'PropIsTrue', value: fst },
            }],
            rule: { kind: 'ImplElim' },
            conclusion: { kind: 'PropIsTrue', value: snd },
        }
    };
}