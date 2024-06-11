import React, { useEffect, useState } from 'react';
import { ProofNode } from './proof-node';
import { ProofTree, annotate_proof_term, ProofTreeRule, Prop } from 'alice';

interface ProofTermVisualizer {
    proofTermString: string;
}

export function ProofTermVisualizer({ proofTermString }: ProofTermVisualizer) {

    const [proofTree, setProofTree] = useState<ProofTree | null>(null);

    useEffect(() => {
        try {
            setProofTree(annotate_proof_term(proofTermString));
        } catch (e) {
            console.log(e);
        }
    }, [proofTermString]);

    return (
        <div style={{ position: 'relative' }}>
            <div>
                {
                    toProofTree(proofTree)
                }
            </div>
        </div>
    );
}

function toProofTree(proofTree: ProofTree | null) {

    if (!proofTree) {
        return null;
    }

    return (
        <div>
            <ProofNode label={printProofRule(proofTree.rule)} content={printProp(proofTree?.conclusion_type)}>
                {
                    proofTree.hypotheses.map(toProofTree)
                }
            </ProofNode>
        </div>
    );

}

function printProofRule(proofTreeRule: ProofTreeRule): string {
    switch (proofTreeRule.kind) {
        case 'AndIntro': return '\\land I';
        case 'AndElimFst': return '\\land E_1';
        case 'AndElimSnd': return '\\land E_2';
        case 'TrueIntro': return '\\top I';
        case 'ImplIntro': return '{\\supset}I^' + proofTreeRule.value;
        case 'ImplElim': return '\\supset E';
        case 'Ident': return proofTreeRule.value ?? '';
        case 'OrIntroFst': return '\\lor I_1';
        case 'OrIntroSnd': return '\\lor I_2';
        case 'OrElim': return `\\lor E^{${proofTreeRule.value[0]}, ${proofTreeRule.value[1]}}`;
        case 'FalsumElim': return '\\bot E';
    }
}

function prettyJoin(arr: string[]): string {

    if (arr.length == 0) {
        return '';
    }

    const elems = [...arr];
    elems.pop();

    let str = elems.join(', ');

    str += arr[arr.length - 1];

    return str;
}

function printProp(prop: Prop): string {
    switch (prop.kind) {
        case 'Any': return '*';
        case 'Atom': return prop.value[0] + (prop.value[1] ?? `(${prettyJoin(prop.value[1])})`);
        case 'And': return printProp(prop.value[0]) + ' ∧ ' + printProp(prop.value[1]);
        case 'Or': return printProp(prop.value[0]) + ' ∨ ' + printProp(prop.value[1]);
        case 'Impl': return printProp(prop.value[0]) + ' ⊃ ' + printProp(prop.value[1]);
        case 'ForAll': return `∀${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'Exists': return `∃${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'True': return '⊤';
        case 'False': return 'False';
    }
}