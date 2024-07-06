import React from 'react';
import { TrashIcon } from '../../app/components/icons/trash-icon';
import { css, cx } from '@emotion/css';
import { useDroppable } from '@dnd-kit/core';

export const TrashOverlayId = 'trash-overlay';

export function TrashOverlay() {

    const { setNodeRef, isOver } = useDroppable({ id: TrashOverlayId });

    return (
        <div className={cx(cssTrashOverlayContainer, { [cssTrashOverlayContainerIsOver]: isOver })} ref={setNodeRef}>
            <div className={cssTrashIcon}>
                <TrashIcon />
            </div>
            <span className={cssText}>Drag over here to delete.</span>
        </div>
    );
}

const cssTrashOverlayContainer = css`
    width: 150px;
    border: 3px solid lightgrey;
    border-style: dashed;
    padding: 10px;
    box-sizing: border-box;
    text-align: center;
`;

const cssTrashOverlayContainerIsOver = css`
    border-color: red;
    background-color: rgba(230, 0, 35, 0.05);
`;

const cssTrashIcon = css`
    width: 75px;
    height: 75px;
    text-align: center;
    margin: auto;
    color: lightgrey;
`;

const cssText = css`
    color: grey;
    text-align: center;
    user-select: none;
`;