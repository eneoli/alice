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

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
            assumptions,
            error,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Cannot apply rule on this node.');
            return;
        }

        const disjunction = await this.promptProp({
            title: 'Enter the disjunction you want to eliminate from.',
            inputPlaceholder: 'A v B',
            assumptions,
            error,
        });

        if (!disjunction) {
            return;
        }

        if (disjunction.kind !== 'Or') {
            error('Your input is not a disjunction.');
            return;
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

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        const {
            selectedProofTreeNodes,
            generateIdentifier,
            generateUniqueNumber,
            assumptions,
            error,
        } = params;

        if (selectedProofTreeNodes.length !== 1) {
            error('Cannot apply this rule on multiple nodes.');
            return;
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            error('Conclusion is not a disjunction.');
            return;
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Or') {
            error('Conclusion is not a disjunction.');
            return;
        }

        const newConclusion = await this.promptProp({
            title: 'Enter new conclusion',
            inputPlaceholder: 'C',
            assumptions,
            error,
        });

        if (!newConclusion) {
            return;
        }

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