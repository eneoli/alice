import { useState } from 'react'
import { VisualProofEditorReasoningContext } from '../lib/visual-proof-editor-reasoning-context';

type ReasoningContextGetter = (id: string) => VisualProofEditorReasoningContext;
type ReasoningContextAdder = (reasoningContext: VisualProofEditorReasoningContext) => void;
type ReasoningContextRemover = (id: string) => void;
type ReasoningContextUpdater = (id: string, newReasoningContext: VisualProofEditorReasoningContext) => void;
type ReasoningContextsUpdater = (reasoningContexts: VisualProofEditorReasoningContext[]) => void;

export interface ReasoingContextHook {
    reasoningContexts: VisualProofEditorReasoningContext[],
    getReasoningContext: ReasoningContextGetter,
    addReasoningContext: ReasoningContextAdder,
    removeReasoningContext: ReasoningContextRemover,
    updateReasoningContext: ReasoningContextUpdater,
    updateReasoningContexts: ReasoningContextsUpdater,
}

export function useReasoningContexts(): ReasoingContextHook {

    const [reasoningContexts, setReasoningContexts] = useState<VisualProofEditorReasoningContext[]>([]);

    const getReasoningContext = (id: string) => {
        const ctx = reasoningContexts.find((ctx) => ctx.id === id);

        if (ctx === undefined) {
            throw new Error('Cannot find reasoning context with id ' + id);
        }

        return ctx;
    };

    const addReasoningContext = (reasoningContext: VisualProofEditorReasoningContext) => {
        const alreadyHasContext = !!reasoningContexts.find((ctx) => ctx.id === reasoningContext.id);
        if (alreadyHasContext) {
            throw new Error('Cannot add already added context with id ' + reasoningContext.id);
        }

        setReasoningContexts([
            ...reasoningContexts,
            reasoningContext,
        ]);
    };

    const removeReasoningContext = (id: string) => {
        setReasoningContexts(reasoningContexts.filter((ctx) => ctx.id !== id));
    }

    const updateReasoningContext = (id: string, newReasoningContext: VisualProofEditorReasoningContext) => {
        setReasoningContexts([
            ...reasoningContexts.filter((ctx) => ctx.id !== id),
            newReasoningContext,
        ]);
    };

    const updateReasoningContexts = (reasoningContexts: VisualProofEditorReasoningContext[]) => {
        setReasoningContexts([...reasoningContexts]);
    };

    return {
        reasoningContexts,
        getReasoningContext,
        addReasoningContext,
        removeReasoningContext,
        updateReasoningContext,
        updateReasoningContexts,
    }
}