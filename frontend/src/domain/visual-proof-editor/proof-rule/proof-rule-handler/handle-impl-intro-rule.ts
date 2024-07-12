import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '../../components/visual-proof-editor-sidebar';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';

export async function handleImplIntroRule({ proofTree, generateIdentifier }: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {

    const { conclusion } = proofTree;

    if (conclusion.kind !== 'PropIsTrue') {
        throw new Error('Conclusion is not an implication.');
    }

    const propConclusion = conclusion.value;

    if (propConclusion.kind != 'Impl') {
        throw new Error('Conclusion is not an implication.');
    }

    const [fst, snd] = propConclusion.value;

    const ident = generateIdentifier();

    return {
        newProofTree: {
            id: proofTree.id,
            premisses: [createEmptyVisualProofEditorProofTreeFromProp(snd)],
            rule: { kind: 'ImplIntro', value: ident },
            conclusion,
        },
        additionalAssumptions: [{
            kind: 'PropIsTrue',
            prop: fst,
            ident,
        }],
    };
}
