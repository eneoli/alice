import React from 'react';
import { css } from '@emotion/css';
import { Button } from 'antd';
import { Assumption, AssumptionContext } from '../proof-rule';
import { print_prop } from 'alice';

interface VisualProofEditorAssumptionViewProps {
    assumptionContexts: AssumptionContext[];
    // onUndoClick: () => void;
    // onRedoClick: () => void;
    onResetClick: () => void;
    onAssumptionClick: (assumptionCtx: AssumptionContext) => void;
}

export function VisualProofEditorAssumptionView(props: VisualProofEditorAssumptionViewProps) {
    const { assumptionContexts, onResetClick, onAssumptionClick } = props;

    return (
        <div className={cssAssumptionContainer}>
            <ul className={cssAssumptionList}>
                {
                    assumptionContexts.map((ctx, i) => (
                        <li key={i} className={cssAssumptionListElement}>
                            <Button onClick={() => onAssumptionClick(ctx)}>
                                {displayAssumption(ctx.assumption)}
                            </Button>
                        </li>
                    ))
                }
            </ul>
            <div className={cssButtonContainer}>
                {/* <Button onClick={onUndoClick}><UndoIcon style={{ width: '20px' }} /></Button> */}
                {/* <Button onClick={onRedoClick}><RedoIcon style={{ width: '20px' }} /></Button> */}
                <Button onClick={onResetClick}
                    type={'primary'}
                    danger={true}>
                    Reset
                </Button>
            </div>
        </div>
    );
}

function displayAssumption(assumption: Assumption) {
    switch (assumption.kind) {
        case 'PropIsTrue': return assumption.ident.name + ': ' + print_prop(assumption.prop);
        case 'Datatype': return assumption.ident.name + ': ' + assumption.datatype;
    }
}

const cssAssumptionContainer = css`
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 5px;
    box-sizing: border-box;
`;

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

const cssButtonContainer = css`
    display: flex;
    * {
        margin-right: 5px;
    }
`;