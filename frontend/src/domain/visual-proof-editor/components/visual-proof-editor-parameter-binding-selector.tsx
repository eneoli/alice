import { css, cx } from '@emotion/css';
import { Identifier, Prop } from 'alice';
import { isEqual } from 'lodash';
import React, { ReactNode, useCallback, useState } from 'react';

interface VisualProofEditorParameterBindingSelectorProps {
    prop: Prop;
    identifier: Identifier;
    onSelect: (selectedParameterIndices: number[]) => void;
}

interface RenderPropResult {
    node: ReactNode,
    newIndex: number,
}

type RenderProp = (prop: Prop, currentIndex: number, bindedIdentifiers: string[]) => RenderPropResult;

export function VisualProofEditorParameterBindingSelector(props: VisualProofEditorParameterBindingSelectorProps) {

    const { prop, identifier, onSelect } = props;

    const [selectedIndices, setSelectedIndices] = useState<number[]>([]);

    const renderProp: RenderProp = (prop: Prop, currentIndex: number, bindedIdentifiers: string[]) => {

        const handlePrimitive = useCallback((primitive: string) => {
            return {
                node: (<span>{primitive}</span>),
                newIndex: currentIndex,
            };
        }, [currentIndex]);

        const handleBinaryConnective = useCallback((connective: string, prop: Prop & { kind: 'And' | 'Or' | 'Impl' }) => {
            const fst = renderProp(prop.value[0], currentIndex, [...bindedIdentifiers]);
            const snd = renderProp(prop.value[1], fst.newIndex, [...bindedIdentifiers]);

            const node = (
                <>
                    {fst.node}
                    <span>{connective}</span>
                    {snd.node}
                </>
            );

            return {
                node,
                newIndex: snd.newIndex,
            };
        }, [currentIndex, bindedIdentifiers]);

        const handleQuantifier = useCallback((quantifier: string, prop: Prop & { kind: 'ForAll' | 'Exists' }) => {
            const body = renderProp(
                prop.value.body,
                currentIndex,
                [...bindedIdentifiers, prop.value.object_ident],
            );

            const node = (
                <>
                    <span>{quantifier}</span>
                    <span>{prop.value.object_ident}</span>
                    <span>:</span>
                    <span>{prop.value.object_type_ident}</span>
                    <span>. </span>
                    <span>{body.node}</span>
                </>
            );

            return {
                node,
                newIndex: body.newIndex,
            };
        }, [currentIndex, bindedIdentifiers]);

        const handleAtom = useCallback((prop: Prop & { kind: 'Atom' }) => {
            const params = prop.value[1];
            const paramNodes = [];

            for (const param of params) {
                const paramName = param.kind === 'Instantiated' ? param.value.name : param.value;

                if (paramName !== identifier.name) {
                    paramNodes.push(<span>{paramName}</span>);
                    continue;
                }

                if (bindedIdentifiers.includes(paramName)) {
                    paramNodes.push(
                        <span title={'This parameter is bound by a quantifier.'}>
                            {paramName}
                        </span>
                    );
                    continue;
                }

                // This should not happen.
                if (param.kind === 'Uninstantiated') {
                    paramNodes.push(
                        <span title={'This parameter is not an instance of your identifier.'}>
                            {paramName}
                        </span>
                    );

                    console.error('Inconsistency error: Uninstantiated free parameter.');
                    continue;
                }

                if (!isEqual(param.value, identifier)) {
                    paramNodes.push(
                        <span title={'This parameter is not an instance of your identifier.'}>
                            {paramName}
                        </span>
                    );
                }

                // Copy index into function scope
                // This is for preventing access to updated index.
                const myIdx = currentIndex;

                const isSelected = selectedIndices.includes(currentIndex);
                const handleClick = () => {
                    const newIndices = isSelected
                        ? selectedIndices.filter((i) => i !== myIdx)
                        : [...selectedIndices, myIdx];

                    newIndices.sort();
                    setSelectedIndices(newIndices);
                    onSelect(newIndices);
                };

                const node = (
                    <span onClick={handleClick}
                        className={cx(
                            cssSelectableParameter,
                            { [cssSelectedParameter]: isSelected },
                        )}>
                        {param.value.name}
                    </span>
                );

                paramNodes.push(node);
                currentIndex = currentIndex + 1;
            }

            const propIdentifier = prop.value[0];
            const node = (
                <>
                    <span>{propIdentifier}</span>
                    <span>(</span>
                    {
                        paramNodes.map((node, i) => (
                            <React.Fragment key={i}>
                                {(i !== 0) && (<span>, </span>)}
                                {node}
                            </React.Fragment>
                        ))
                    }
                    <span>)</span>
                </>
            );

            return {
                newIndex: currentIndex,
                node,
            };
        }, [identifier, selectedIndices]);

        switch (prop.kind) {
            case 'True': return handlePrimitive('⊤');
            case 'False': return handlePrimitive('⊥');
            case 'Atom': return handleAtom(prop);
            case 'And': return handleBinaryConnective('∧', prop);
            case 'Or': return handleBinaryConnective('∨', prop);
            case 'Impl': return handleBinaryConnective('⊃', prop);
            case 'ForAll': return handleQuantifier('∀', prop);
            case 'Exists': return handleQuantifier('∃', prop);
        }
    }

    const node = renderProp(prop, 0, []).node;

    return (
        <div className={cssParameterBindingSelectorContainer}>
            {node}
        </div>
    );
}

const cssParameterBindingSelectorContainer = css`
    font-size: 2em;
    user-select: none;
    *  {
        user-select: none;
    }
`;

const cssSelectableParameter = css`
    cursor: pointer;
    color: grey;
`;

const cssSelectedParameter = css`
    color: #006af5;
`;