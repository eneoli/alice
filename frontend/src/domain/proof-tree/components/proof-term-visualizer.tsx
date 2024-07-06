import React, { Fragment, useEffect, useState } from 'react';
import { ProofNode } from './proof-node';
import { ProofTree, ProofTreeRule, Prop, verify } from 'alice';

interface ProofTermVisualizer {
    prop: string;
    proofTermString: string;
}

export function ProofTermVisualizer({ prop, proofTermString }: ProofTermVisualizer) {

    const [proofTree, setProofTree] = useState<ProofTree | null>(null);

    useEffect(() => {
        try {
            setProofTree(verify(prop, proofTermString));
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

    const conclusion = proofTree.conclusion.kind === 'PropIsTrue' ? printProp(proofTree.conclusion.value) : proofTree.conclusion.value[0] + ':' + proofTree.conclusion.value[1];

    return (
        <div>
            <ProofNode rule={printProofRule(proofTree.rule)} content={conclusion}>
                {
                    proofTree.premisses.map((child: ProofTree, i: number) => <Fragment key={i}>{toProofTree(child)}</Fragment>)
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
        case 'ForAllIntro': return '\\forall I^' + proofTreeRule.value;
        case 'ForAllElim': return '\\forall E';
        case 'ExistsIntro': return '\\exists I';
        case 'ExistsElim': return `\\exists E^{${proofTreeRule.value[0]}, ${proofTreeRule.value[1]}}`;
    }
}

function printProp(prop: Prop): string {
    switch (prop.kind) {
        case 'Atom': return prop.value[0] + (prop.value[1].length > 0 ? `(${prop.value[1].join(', ')})` : '');
        case 'And': return printProp(prop.value[0]) + ' ∧ ' + printProp(prop.value[1]);
        case 'Or': return printProp(prop.value[0]) + ' ∨ ' + printProp(prop.value[1]);
        case 'Impl': return printProp(prop.value[0]) + ' ⊃ ' + printProp(prop.value[1]);
        case 'ForAll': return `∀${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'Exists': return `∃${prop.value.object_ident}:${prop.value.object_type_ident}. ` + printProp(prop.value.body);
        case 'True': return '⊤';
        case 'False': return 'False';
    }
}