import { css } from '@emotion/css';
import { Progress } from 'antd';
import React, { ReactNode } from 'react';

export type TutorPropositionSolutionStatusStatus = 'solved' | 'solvable' | 'unsolvable' | 'unknown';

interface TutorPropositionSolutionStatusProps {
    status: TutorPropositionSolutionStatusStatus;
    percentage: number;
}

export function TutorPropositionSolutionStatus(props: TutorPropositionSolutionStatusProps) {
    const { status, percentage } = props;

    const statusText = getText(status);

    return (
        <div className={cssStatusContainer}>
            <Progress
                type="circle"
                size={90}
                strokeColor={'#1677ff'}
                percent={percentage}
            />
            <span className={cssStatusText}>
                {statusText}
            </span>
        </div>
    );
}

function getText(status: TutorPropositionSolutionStatusStatus): ReactNode {
    switch (status) {
        case 'solved': return (
            <span>
                <span style={{color: '#4BB543'}}>You did it!</span>
                <br/>
                You proved the proposition!
            </span>
        );
        case 'solvable': return (
            <span>
                Alice thinks <span style={{ color: '#12DC19' }}>you can prove</span> this proposition.
            </span>
        );
        case 'unsolvable': return (
            <span>
                Alice thinks you <span style={{ color: '#ED2836' }}>can&apos;t prove</span> this proposition.
            </span>
        );
        case 'unknown': return (
            <span>
                Alice <span style={{ color: '#FF8343' }}>doesn&apos;t know</span> whether you can prove this proposition.
            </span>
        );
    }
}

const cssStatusContainer = css`
    display: flex;
    justify-content: center;
    gap: 10px;
    align-items: center;
    flex-direction: row;
`;

const cssStatusText = css`
    text-align: center;
    width: 90%;
    font-size: 1.3em;
    font-weight: bold;
    color: #fefefe;
    display: block;
`;