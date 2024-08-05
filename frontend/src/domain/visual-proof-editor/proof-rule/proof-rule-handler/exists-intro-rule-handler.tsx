import { bind_identifier, get_free_parameters, Identifier, instantiate_free_parameter, Prop } from 'alice';
import Swal from 'sweetalert2';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import { ProofRuleHandler } from './proof-rule-handler';
import { v4 } from 'uuid';
import { createEmptyVisualProofEditorProofTreeFromProp, createEmptyVisualProofEditorProofTreeFromTypeJudgment } from '../../lib/visual-proof-editor-proof-tree';
import withReactContent from 'sweetalert2-react-content';
import React from 'react';
import { VisualProofEditorParameterBindingSelector } from '../../components/visual-proof-editor-parameter-binding-selector';
import { createIdentifierGenerator } from './create-identifier-generator';

const ReactSwal = withReactContent(Swal);

export class ExistsIntroRuleHandler extends ProofRuleHandler {
    public getLatexCode(): string {
        return `
            \\begin{prooftree}
                \\AxiomC{$a: \\tau$}
                \\AxiomC{$A(a)$}
                \\RightLabel{$\\exists I$}
                \\BinaryInfC{$\\exists x: \\tau. A(x)$}
            \\end{prooftree}
        `;
    }

    protected async handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes } = params;

        if (selectedProofTreeNodes.length !== 1) {
            throw new Error('Cannot apply this rule on multiple nodes.');
        }

        const { proofTree, reasoningContextId } = selectedProofTreeNodes[0];
        const { conclusion } = proofTree;

        if (conclusion.kind !== 'PropIsTrue') {
            throw new Error('Conclusion is not an existential quantification');
        }

        const propConclusion = conclusion.value;

        if (propConclusion.kind !== 'Exists') {
            throw new Error('Conclusion is not an existential quantification');
        }

        // ask for instantiation identifier
        const instantiationIdentifier = await Swal.fire({
            title: 'Enter instantiation identifier',
            input: 'text',
            inputLabel: 'Identifier',
            inputPlaceholder: 'a',
            showCloseButton: true,
        });

        const { object_ident, object_type_ident, body } = propConclusion.value;

        // TODO
        const instantiated_body = instantiate_free_parameter(body, object_ident, { name: instantiationIdentifier.value, unique_id: 0 });

        return {
            additionalAssumptions: [],
            removedReasoingContextIds: [],
            newReasoningContexts: [],
            proofTreeChanges: [{
                newProofTree: {
                    id: proofTree.id,
                    premisses: [
                        createEmptyVisualProofEditorProofTreeFromTypeJudgment(instantiationIdentifier.value, object_type_ident),
                        createEmptyVisualProofEditorProofTreeFromProp(instantiated_body),
                    ],
                    rule: { kind: 'ExistsIntro' },
                    conclusion,
                },
                nodeId: proofTree.id,
                reasoningContextId,
            }]
        };
    }

    protected async handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        const { selectedProofTreeNodes } = params;

        if (selectedProofTreeNodes.length !== 2) {
            throw new Error('Need exactly two nodes to form an existential proposition.');
        }

        const [fst, snd] = selectedProofTreeNodes;

        const fstIsTypeJudgment = fst.proofTree.conclusion.kind === 'TypeJudgement';
        const sndIsTypeJudgment = snd.proofTree.conclusion.kind === 'TypeJudgement';

        if (fstIsTypeJudgment && sndIsTypeJudgment) {
            throw new Error('Cannot combine two type judgments to form an existential propositon.');
        }

        if (!fstIsTypeJudgment && !sndIsTypeJudgment) {
            throw new Error('Cannot combine two propositions to form an existential proposition.');
        }

        const typeJudgmentNode = fstIsTypeJudgment ? fst : snd;
        const propJudgmentNode = fstIsTypeJudgment ? snd : fst;

        const prop = propJudgmentNode.proofTree.conclusion.value as Prop;
        const identifier = (typeJudgmentNode.proofTree.conclusion.value as [Identifier, string])[0];
        const typeName = (typeJudgmentNode.proofTree.conclusion.value as [Identifier, string])[1];

        // ask to bind identifiers

        let parameterIndices: Uint32Array = new Uint32Array();
        const prompt = await ReactSwal.fire({
            title: 'Click on the parameters you want to bind.',
            showCloseButton: true,
            html: (
                <VisualProofEditorParameterBindingSelector
                    prop={prop}
                    identifier={identifier}
                    onSelect={(s) => { parameterIndices = new Uint32Array(s); }}
                />
            )
        });

        if (prompt.isDismissed || prompt.isDenied) {
            return this.createEmptyProofRuleHandlerResult();
        }

        const freeParams = get_free_parameters(prop);
        const freeParamNames = freeParams
            .map((ident) => ident.kind === 'Instantiated' ? ident.value.name : ident.value)

        const generateLocalIdentifier = createIdentifierGenerator('xyzuvwabcdefghijklmnopqrst'.split(''));

        let bindIdentifier = generateLocalIdentifier();

        while (freeParamNames.includes(bindIdentifier)) {
            bindIdentifier = generateLocalIdentifier();
        }

        const bindedProp = bind_identifier(
            prop,
            { kind: 'Exists' },
            identifier,
            parameterIndices,
            bindIdentifier,
            typeName,
        );

        return {
            additionalAssumptions: [],
            proofTreeChanges: [],
            removedReasoingContextIds: [fst.reasoningContextId, snd.reasoningContextId],
            newReasoningContexts: [{
                id: v4(),
                selectedNodeId: null,
                isDragging: false,
                x: 0,
                y: 0,
                proofTree: {
                    id: v4(),
                    premisses: [typeJudgmentNode.proofTree, propJudgmentNode.proofTree],
                    rule: { kind: 'ExistsIntro' },
                    conclusion: { kind: 'PropIsTrue', value: bindedProp },
                }
            }],
        };
    }
}
