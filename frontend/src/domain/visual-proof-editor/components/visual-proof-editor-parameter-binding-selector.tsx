import { css, cx } from '@emotion/css';
import { Identifier, Prop, PropParameter } from 'alice';
import { isEqual } from 'lodash';
import React, { ReactNode, useCallback, useEffect, useState } from 'react';

interface VisualProofEditorParameterBindingSelectorProps {
    prop: Prop;
    identifier?: Identifier;
    onSelect: (identifier: Identifier, selectedParameterIndices: number[]) => void;
}

interface RenderPropResult {
    node: ReactNode,
    newIndex: number,
}

type RenderProp = (prop: Prop, currentIndex: number, bindedIdentifiers: string[]) => RenderPropResult;

export function VisualProofEditorParameterBindingSelector(props: VisualProofEditorParameterBindingSelectorProps) {

    const { prop, onSelect } = props;

    const [selectedIndices, setSelectedIndices] = useState<number[]>([]);

    const [identifier, setIdentifier] = useState<Identifier | null>(null);

    useEffect(() => {
        setIdentifier(props.identifier || null);
        setSelectedIndices([]);
    }, [props.identifier]);

    useEffect(() => {
        if (selectedIndices.length === 0 && props.identifier === undefined) {
            setIdentifier(null);
        }
    }, [selectedIndices]);

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

                // allow to select any parameter as identifier
                if (!identifier) {
                    if (param.kind === 'Uninstantiated') {
                        paramNodes.push(
                            <span title={'This parameter is already bound.'}>
                                {paramName}
                            </span>
                        );

                        currentIndex++;
                        continue;
                    }

                    const handleIdentifierSelect = () => {
                        setIdentifier(param.value);

                        // get relative index
                        const allParams = getAllParameters(props.prop);
                        const relativeIndex = allParams
                            .slice(0, currentIndex + 1)
                            .filter(isEqual.bind(param.value))
                            .length;

                        setSelectedIndices([relativeIndex]);
                        onSelect(param.value, [relativeIndex]);
                    };

                    paramNodes.push(
                        <span onClick={handleIdentifierSelect} className={cssSelectableParameter}>
                            {paramName}
                        </span>
                    );

                    currentIndex++;
                    continue;
                }

                if (paramName !== identifier.name.toString()) {
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
                    onSelect(identifier, newIndices);
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

function getAllParameters(prop: Prop): PropParameter[] {
    switch (prop.kind) {
        case 'Atom': return [...prop.value[1]];
        case 'And':
        case 'Or':
        case 'Impl':
            return [...getAllParameters(prop.value[0]), ...getAllParameters(prop.value[1])];
        case 'ForAll':
        case 'Exists':
            return getAllParameters(prop.value.body);
        case 'True': return [];
        case 'False': return [];
    }
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