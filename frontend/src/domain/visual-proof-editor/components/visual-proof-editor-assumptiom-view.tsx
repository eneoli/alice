import React from 'react';
import { printProp } from '../../../util/print-prop';
import { css } from '@emotion/css';
import { Assumption } from './visual-proof-editor-sidebar';
import { Button } from 'antd';

interface VisualProofEditorAssumptionViewProps {
    assumptions: Assumption[];
    onResetClick: () => void;
    onAssumptionClick: (assumption: Assumption) => void;
}

export function VisualProofEditorAssumptionView(props: VisualProofEditorAssumptionViewProps) {
    const { assumptions, onResetClick, onAssumptionClick } = props;

    return (
        <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between', width: '100%' }}>
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
            <Button onClick={onResetClick} type={'primary'} danger={true} className={cssResetButton}>Reset</Button>
        </div>
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
    list-style: none;
    height: 32px;
    padding: none;
    position: relative;
    z-index: 1000;
`;

const cssAssumptionListElement = css`
    display: inline;
    margin-right: 5px;
`;

const cssResetButton = css`
    margin-right: 35px;
    margin-top: 20px;
`;