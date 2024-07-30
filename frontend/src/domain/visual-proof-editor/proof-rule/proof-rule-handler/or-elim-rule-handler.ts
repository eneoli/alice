import Swal from 'sweetalert2';
import { Identifier, Prop } from 'alice';
import { v4 } from 'uuid';
import { VisualProofEditorRuleHandlerParams, ProofRuleHandlerResult, AssumptionContext } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { createEmptyVisualProofEditorProofTreeFromProp } from '../../lib/visual-proof-editor-proof-tree';

export class OrElimRuleHandler extends ProofRuleHandler {

    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$A \\lor B$}
                \\AxiomC{[$A$]}
                \\noLine
                \\UnaryInfC{$C$}
                \\AxiomC{[$B$]}
                \\noLine
                \\UnaryInfC{$C$}
                \\RightLabel{$\\lor E$}
                \\TrinaryInfC{$C$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
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
            generateUniqueNumber,
            reasoningContextId,
        });

        const fstIdent = additionalAssumptions[0].assumption.ident;
        const sndIdent = additionalAssumptions[1].assumption.ident;

        return {
            additionalAssumptions,
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    ...proofTree,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromProp(disjunction),
                        createEmptyVisualProofEditorProofTreeFromProp(conclusion.value),
                        createEmptyVisualProofEditorProofTreeFromProp(conclusion.value),
                    ],
                    rule: { kind: 'OrElim', value: [fstIdent.name, sndIdent.name] },
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }],
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
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
            generateUniqueNumber,
            reasoningContextId,
        });

        const fstIdent = additionalAssumptions[0].assumption.ident;
        const sndIdent = additionalAssumptions[1].assumption.ident;

        return {
            additionalAssumptions,
            newReasoningContexts: [],
            removedReasoingContextIds: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: nodeId,
                    premisses: [
                        { ...proofTree },
                        createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                        createEmptyVisualProofEditorProofTreeFromProp(newConclusion),
                    ],
                    rule: { kind: 'OrElim', value: [fstIdent.name, sndIdent.name] },
                    conclusion: { kind: 'PropIsTrue', value: newConclusion },
                },
                reasoningContextId,
                nodeId: proofTree.id,
            }],
        };
    }
}

interface CreateProofRuleHandlerResultAssumptionContextsParams {
    head: Prop,
    nodeId: string,
    reasoningContextId: string,
    generateIdentifier: () => string,
    generateUniqueNumber: () => number,
}

function createProofRuleHandlerResultAssumptionContexts(params: CreateProofRuleHandlerResultAssumptionContextsParams): AssumptionContext[] {
    const {
        head,
        nodeId,
        reasoningContextId,
        generateIdentifier,
        generateUniqueNumber,
    } = params;

    const fstIdent: Identifier = {
        name: generateIdentifier(),
        unique_id: generateUniqueNumber(),
    };

    const sndIdent: Identifier = {
        name: generateIdentifier(),
        unique_id: generateUniqueNumber(),
    };

    if (head.kind !== 'Or') {
        throw new Error('Expected head to be disjunction.');
    }

    const [fst, snd] = head.value;

    return [
        { assumption: { kind: 'PropIsTrue', ident: fstIdent, prop: fst }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
        { assumption: { kind: 'PropIsTrue', ident: sndIdent, prop: snd }, owningReasoningCtxId: reasoningContextId, owningNodeId: nodeId },
    ];
}