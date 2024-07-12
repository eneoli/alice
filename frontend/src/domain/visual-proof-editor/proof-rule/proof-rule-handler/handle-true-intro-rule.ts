import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';

export async function handleTrueIntroRule({ proofTree }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not truth.');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind !== 'True') {
        throw new Error('Conclusion is not truth.');
    }

    return {
        additionalAssumptions: [],
        newProofTree: {
            ...proofTree,
            rule: { kind: 'TrueIntro' },
        }
    };
}