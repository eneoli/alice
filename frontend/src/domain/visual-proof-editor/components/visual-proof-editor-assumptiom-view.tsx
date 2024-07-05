import React from 'react';
import { printProp } from '../../../util/print-prop';
import { css } from '@emotion/css';
import { Assumption } from './visual-proof-editor-sidebar';
import { Button } from 'antd';

interface VisualProofEditorAssumptionViewProps {
    assumptions: Assumption[];
    onAssumptionClick: (assumption: Assumption) => void;
}

export function VisualProofEditorAssumptionView(props: VisualProofEditorAssumptionViewProps) {
    const { assumptions, onAssumptionClick } = props;

    return (
        <ul className={cssAssumptionList}>
            {
                assumptions.map((assumption, i) => (
                    <li key={i} className={cssAssumptionListElement}>
                        <Button onClick={() => onAssumptionClick(assumption)}>
                            {displayAssumption(assumption)}
                        </Button>
                    </li>
                ))
            }
        </ul>
    );
}

function displayAssumption(assumption: Assumption) {
    switch (assumption.kind) {
        case 'PropIsTrue': return assumption.ident + ': ' + printProp(assumption.prop);
        case 'Datatype': return assumption.ident + ': ' + assumption.datatype;
    }
}

const cssAssumptionList = css`
    display: inline-block;
    list-decoration: none;
    padding: none;
    position: relative;
    z-index: 1000;
`;

const cssAssumptionListElement = css`
    display: inline;
    margin-right: 5px;
`;