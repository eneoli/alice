import { get_free_parameters, Identifier, instantiate_free_parameter, parse_prop, Prop } from 'alice';
import { AssumptionContext, ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';
import Swal from 'sweetalert2';
import { uniq } from 'lodash';
import withReactContent from 'sweetalert2-react-content';
import React from 'react';
import { VisualProofEditorParameterIdentifierSelector } from '../../components/visual-proof-editor-parameter-identifier-selector';

const ReactSwal = withReactContent(Swal);

interface PromptPropOptions {
    title: string;
    inputPlaceholder?: string;
    assumptions: AssumptionContext[];
    error: (msg: string) => void;
}

export abstract class ProofRuleHandler {

    public abstract getLatexCode(): string;

    protected abstract handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined>;

    protected abstract handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined>;

    protected async promptProp(options: PromptPropOptions): Promise<Prop | null> {
        const {
            title,
            inputPlaceholder,
            assumptions,
            error,
        } = options;

        let prop = (await Swal.fire({
            title,
            inputPlaceholder,
            input: 'text',
            showCloseButton: true,
        })).value;

        if (!prop) {
            return null;
        }

        prop = this.parseProp(prop);

        let params = get_free_parameters(prop);
        params = uniq(params);

        const needSelectionNames = [];
        for (const param of params) {
            if (param.kind !== 'Uninstantiated') {
                continue;
            }

            const paramName = param.value;

            const paramAssumptions = assumptions
                .filter((assumption) => assumption.assumption.ident.name === paramName);

            if (paramAssumptions.length === 0) {
                error(`Unknown identifier: ${paramName}`);
                return null;
            }

            const datatypeParamAssumptions = paramAssumptions
                .filter((assumption) => assumption.assumption.kind === 'Datatype');

            if (datatypeParamAssumptions.length === 0) {
                error(`Not a value: ${paramName}`);
                return null;
            }

            if (datatypeParamAssumptions.length > 1) {
                needSelectionNames.push(paramName);
                continue;
            }

            prop = instantiate_free_parameter(prop, paramName, datatypeParamAssumptions[0].assumption.ident);
        }

        if (needSelectionNames.length === 0) {
            return prop;
        }

        // prompt for user selection
        const selectionOptions = needSelectionNames
            .reduce((accu, name) => {
                accu[name] = assumptions
                    .filter((assumptions) => assumptions.assumption.ident.name === name)
                    .map((assumption) => assumption.assumption.ident);

                return accu;
            }, {} as { [name: string]: Identifier[] });

        let finalProp: Prop | null = null;
        await ReactSwal.fire({
            title: 'Please further specify which identifier you mean.',
            html: React.createElement(VisualProofEditorParameterIdentifierSelector, {
                prop,
                options: selectionOptions,
                onAllSelect: (prop) => { finalProp = prop; },
                getPopupContainer: () => Swal.getContainer() || document.body,
            }),
        });

        return finalProp;
    }

    protected parseProp(prop: string): Prop {
        try {
            return parse_prop(prop);
        } catch (_) {
            throw new Error('Failed to parse prop.');
        }
    }

    protected createEmptyProofRuleHandlerResult(): ProofRuleHandlerResult {
        return {
            additionalAssumptions: [],
            newReasoningContexts: [],
            proofTreeChanges: [],
            removedReasoingContextIds: []
        };
    }

    protected allNodesAreLeafs(params: VisualProofEditorRuleHandlerParams): boolean {
        return params.selectedProofTreeNodes.every((node) => node.isLeaf);
    }

    protected allNodesAreRoots(params: VisualProofEditorRuleHandlerParams): boolean {
        return params.selectedProofTreeNodes.every((node) => node.isRoot);
    }

    protected allNodesHaveRules(params: VisualProofEditorRuleHandlerParams): boolean {
        return params.selectedProofTreeNodes.every((node) => node.proofTree.rule !== null);
    }

    protected isSingleNodeSelected(params: VisualProofEditorRuleHandlerParams): boolean {
        return params.selectedProofTreeNodes.length === 1;
    }

    public willReasonDownwards(params: VisualProofEditorRuleHandlerParams): boolean {
        return this.allNodesAreRoots(params) && this.allNodesHaveRules(params);
    }

    public willReasonUpwards(params: VisualProofEditorRuleHandlerParams): boolean {
        return (
            !this.willReasonDownwards(params) &&
            this.isSingleNodeSelected(params) &&
            this.allNodesAreLeafs(params)
        );
    }

    public async handleRule(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult | undefined> {
        if (this.willReasonDownwards(params)) {
            return this.handleRuleDownards(params);
        }

        if (this.willReasonUpwards(params)) {
            return this.handleRuleUpwards(params);
        }

        throw new Error('Cannot reason on this node.');
    }
}