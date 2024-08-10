import { get_free_parameters, Identifier, instantiate_free_parameter_by_index, Prop } from 'alice';
import { Select, Space } from 'antd';
import React, { Fragment, ReactElement, useCallback, useEffect, useRef, useState } from 'react';

interface VisualProofEditorParameterIdentifierSelectorProps {
    prop: Prop;
    options: { [name: string]: Identifier[] },
    onAllSelect: (prop: Prop) => void;
    getPopupContainer?: () => HTMLElement;
}

interface RenderPropResult {
    node: ReactElement;
    newIndex: number;
}

type RenderProp = (prop: Prop, index: number, boundIdentifiers: string[]) => RenderPropResult;

export function VisualProofEditorParameterIdentifierSelector(props: VisualProofEditorParameterIdentifierSelectorProps) {
    const {
        prop,
        options,
        onAllSelect,
        getPopupContainer,
    } = props;

    const containerRef = useRef<HTMLDivElement | null>(null);
    const [paramMapping, setParamMapping] = useState<{ [idx: number]: number } | null>(null);

    useEffect(() => {
        const selectableParamSize = get_free_parameters(prop)
            .filter((param) => param.kind === 'Uninstantiated')
            .length;

        setParamMapping(Object.assign({}, Array(selectableParamSize).fill(0)));
    }, [prop]);

    useEffect(() => {

        if (!paramMapping) {
            return;
        }

        const params = get_free_parameters(prop);
        let instantiatedProp = { ...prop };
        for (const [paramIndex, identifierIndex] of Object.entries(paramMapping)) {
            const param = params[parseInt(paramIndex)];
            const paramName = param.kind === 'Instantiated' ? param.value.name : param.value;

            instantiatedProp = instantiate_free_parameter_by_index(
                instantiatedProp,
                parseInt(paramIndex),
                options[paramName][identifierIndex],
            );
        }

        onAllSelect(instantiatedProp);
    }, [paramMapping]);

    const renderProp: RenderProp = (prop: Prop, index: number, boundIdentifiers: string[]) => {

        const handlePrimitive = useCallback((symbol: string, index: number) => {
            return {
                node: (<span>{symbol}</span>),
                newIndex: index,
            };
        }, []);

        const handleBinaryConnective = useCallback((symbol: string, prop: Prop & { kind: 'And' | 'Or' | 'Impl' }, index: number, boundIdentifiers: string[]) => {
            const fst = renderProp(prop, index, [...boundIdentifiers]);
            const snd = renderProp(prop, fst.newIndex, boundIdentifiers);

            const node = (
                <>
                    {fst.node}
                    <span>
                        {symbol}
                    </span>
                    {snd.node}
                </>
            );

            return {
                node,
                newIndex: snd.newIndex,
            };
        }, []);

        const handleQuantifier = useCallback((symbol: string, prop: Prop & { kind: 'ForAll' | 'Exists' }, index: number, boundIdentifiers: string[]) => {
            const body = renderProp(
                prop.value.body,
                index,
                [...boundIdentifiers, prop.value.object_ident],
            );

            const node = (
                <>
                    <span>{symbol}</span>
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
        }, []);

        const handleAtom = useCallback((prop: Prop & { kind: 'Atom' }, index: number, boundIdentifiers: string[]) => {
            const params = prop.value[1];

            const paramNodes = [];
            for (const param of params) {
                const paramIndex = index;
                const paramName = param.kind === 'Instantiated' ? param.value.name : param.value;

                if (param.kind === 'Instantiated' || boundIdentifiers.includes(paramName)) {
                    paramNodes.push(<span>{paramName}</span>);

                    index++;
                    continue;
                }

                const identOptions = options[paramName];

                if (!identOptions || identOptions.length === 0) {
                    throw new Error(`No options for name ${paramName}.`);
                }

                const selectionOptions = identOptions
                    .map((option, i) => ({
                        label: `${option.name} - ${option.unique_id}`,
                        value: i,
                    }));

                const handleSelect = (idx: number) => {
                    setParamMapping({
                        ...paramMapping,
                        [paramIndex]: idx,
                    });
                };

                const node = (
                    <Space wrap>
                        <Select
                            getPopupContainer={getPopupContainer}
                            onChange={handleSelect}
                            defaultValue={0}
                            options={selectionOptions}
                        />
                    </Space>
                );

                paramNodes.push(node);
                index++;
            }

            const propName = prop.value[0];

            const node = (
                <>
                    <span>{propName}</span>
                    {paramNodes.length > 0 && (
                        <>
                            <span>(</span>
                            {
                                paramNodes.map((node, i) => (
                                    <Fragment key={i}>
                                        {i > 0 && <span>, </span>}
                                        {node}
                                    </Fragment>
                                ))
                            }
                            <span>)</span>
                        </>
                    )}
                </>
            );

            return {
                node,
                newIndex: index,
            }
        }, []);

        switch (prop.kind) {
            case 'True': return handlePrimitive('⊤', index);
            case 'False': return handlePrimitive('⊥', index);
            case 'And': return handleBinaryConnective('∧', prop, index, boundIdentifiers);
            case 'Or': return handleBinaryConnective('∨', prop, index, boundIdentifiers);
            case 'Impl': return handleBinaryConnective('⊃', prop, index, boundIdentifiers);
            case 'ForAll': return handleQuantifier('∀', prop, index, boundIdentifiers);
            case 'Exists': return handleQuantifier('∃', prop, index, boundIdentifiers);
            case 'Atom': return handleAtom(prop, index, boundIdentifiers);
        }
    };

    return (
        <div ref={containerRef}>
            {renderProp(prop, 0, []).node}
        </div>
    );
}