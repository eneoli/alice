import { ProofTreeConclusion, Prop } from 'alice';
import { VisualProofEditorProofTree } from './visual-proof-editor-proof-tree';
import { createEmptyVisualProofEditorProofTreeFromConclusion, createEmptyVisualProofEditorProofTreeFromProp } from '../../../util/create-visual-proof-editor-empty-proof-tree';
import { v4 } from 'uuid';

export interface VisualProofEditorReasoningContext {
    id: string;
    proofTree: VisualProofEditorProofTree;
    selectedNodeId: string | null;
    isDragging: boolean;
    x: number;
    y: number;
}

export function createEmptyVisualProofEditorReasoningContextFromProp(prop: Prop): VisualProofEditorReasoningContext {
    return {
        id: v4(),
        selectedNodeId: null,
        proofTree: createEmptyVisualProofEditorProofTreeFromProp(prop),
        isDragging: false,
        x: 0,
        y: 0,
    };
}

export function createEmptyVisualProofEditorReasoningContextFromConclusion(conclusion: ProofTreeConclusion): VisualProofEditorReasoningContext {
    return {
        id: v4(),
        selectedNodeId: null,
        proofTree: createEmptyVisualProofEditorProofTreeFromConclusion(conclusion),
        isDragging: false,
        x: 0,
        y: 0,
    };
}
