import { parse_prop, Prop } from 'alice';
import { ProofRuleHandlerResult, VisualProofEditorRuleHandlerParams } from '..';

export abstract class ProofRuleHandler {
    protected abstract handleRuleUpwards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult>;

    protected abstract handleRuleDownards(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult>;

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

    public async handleRule(params: VisualProofEditorRuleHandlerParams): Promise<ProofRuleHandlerResult> {
        if (this.willReasonDownwards(params)) {
            return this.handleRuleDownards(params);
        }

        if (this.willReasonUpwards(params)) {
            return this.handleRuleUpwards(params);
        }

        throw new Error('Cannot reason on this node.');
    }
}