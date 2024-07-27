import Swal from 'sweetalert2';
import { Prop } from 'alice';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../../../util/create-visual-proof-editor-empty-proof-tree';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult, AssumptionContext } from '..';
import { ProofRuleHandler } from './proof-rule-handler';

export class OrElimRuleHandler extends ProofRuleHandler {
    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree, reasoningContextId, generateIdentifier } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Cannot apply rule on this node.');
        }

        // ask for disjunction
        let disjunction = (await Swal.fire({
            title: 'Enter the disjunction you want to eliminate from.',
            input: 'text',
            inputPlaceholder: 'A v B',
            showCloseButton: true,
        })).value;

        if (!disjunction) {
            return this.createEmptyProofRuleHandlerResult();
        }

        disjunction = this.parseProp(disjunction);

        if (disjunction.kind !== 'Or') {
            throw new Error('Your input is not a disjunction.');
        }

        const additionalAssumptions = createProofRuleHandlerResultAssumptionContexts({
            head: disjunction,
            nodeId: proofTree.id,
            generateIdentifier,
            reasoningContextId,
        });

        const fstIdent = additionalAssumptions[0].assumption.ident;
        const sndIdent = additionalAssumptions[1].assumption.ident;

        return {
            additionalAssumptions,
            newProofTree: {
                ...proofTree,
                premisses: [
                    createEmptyVisualProofEditorProofTreeFromProp(disjunction),
                    createEmptyVisualProofEditorProofTreeFromProp(conclusion.value),
                    createEmptyVisualProofEditorProofTreeFromProp(conclusion.value),
                ],
                rule: { kind: 'OrElim', value: [fstIdent, sndIdent] },
            }
        };

    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { proofTree, reasoningContextId, generateIdentifier } = params;
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not a disjunction.');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Or') {
            throw new Error('Conclusion is not a disjunction.');
        }

        // ask for new conclusion
        const newConclusionPromptResult = await Swal.fire({
            title: 'Enter new conclusion',
            input: 'text',
            inputPlaceholder: 'C',
            showCloseButton: true,
        });

        if (!newConclusionPromptResult.isConfirmed) {
            return this.createEmptyProofRuleHandlerResult();
        }

        const newConclusion = this.parseProp(newConclusionPromptResult.value);

        const nodeId = v4();
        
        const additionalAssumptions = createProofRuleHandlerResultAssumptionContexts({
            head: conclusion.value,
            nodeId,
            generateIdentifier,
            reasoningContextId,
        });

        const fstIdent = additionalAssumptions[0].assumption.ident;
        const sndIdent = additionalAssumptions[1].assumption.ident;

        return {
            additionalAssumptions,
            newProofTree: {
                id: nodeId,
                premisses: [
                    { ...proofTree },
                    createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                    createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                ],
                rule: { kind: 'OrElim', value: [fstIdent, sndIdent] },
                conclusion: { kind: 'PropIsTrue', value: newConclusion },
            }
        };
    }
}

interface CreateProofRuleHandlerResultAssumptionContextsParams {
    head: Prop,
    nodeId: string,
    reasoningContextId: string,
    generateIdentifier: () => string,
}

function createProofRuleHandlerResultAssumptionContexts(params: CreateProofRuleHandlerResultAssumptionContextsParams): AssumptionContext[] {
    const { head, nodeId, reasoningContextId, generateIdentifier } = params;
    const fstIdent = generateIdentifier();
    const sndIdent = generateIdentifier();

    if (head.kind !== 'Or') {
        throw new Error('Expected head to be disjunction.');
    }

    const [fst, snd] = head.value;

    return [
        { assumption: { kind: 'PropIsTrue', ident: fstIdent, prop: fst }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
        { assumption: { kind: 'PropIsTrue', ident: sndIdent, prop: snd }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
    ];
}