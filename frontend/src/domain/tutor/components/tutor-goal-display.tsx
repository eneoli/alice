import { css } from '@emotion/css';
import { ProofTreeConclusion } from 'alice';
import React from 'react';
import { printProofTreeConclusion } from '../../../util/print-proof-tree-conclusion';

export interface TutorGoalDisplayGoal {
    proofTreeConclusion: ProofTreeConclusion,
    hint: string,
}

interface TutorGoalDisplayProps {
    goals: TutorGoalDisplayGoal[];
}

export function TutorGoalDisplay(props: TutorGoalDisplayProps) {
    const { goals } = props;

    if (goals.length === 0) {
        return (
            <div className={cssGoalDisplayContainer}>
                <span style={{ textAlign: 'center', width: '100%' }} className={cssTitle}>
                    🎉 All goals closed!
                </span>
            </div>
        );
    }

    return (
        <div className={cssGoalDisplayContainer}>
            <span className={cssTitle}>
                🚩 You have <span className={cssTitleGoalNumber}>{goals.length}</span> open {goals.length === 1 ? ('goal') : ('goals')}:
            </span>

            <ul className={cssGoalList}>
                {
                    goals.map((goal, i) => (
                        <li key={i}>
                            <span>
                                ⊢ <span className={cssProp}>{printProofTreeConclusion(goal.proofTreeConclusion)}</span>
                                <br />
                                <br />
                                Hint: <span className={cssHint}>{goal.hint}</span>
                            </span>
                        </li>
                    ))
                }
            </ul>
        </div>
    );
}

const cssGoalDisplayContainer = css`
    display: flex;
    flex-direction: column;
    align-items: flex-start;
`;

const cssTitle = css`
    color: white;
    font-size: 1.5em;
`;

const cssTitleGoalNumber = css`
    font-weight: bold;
    color: #6CB4EE;
`;

const cssGoalList = css`
    width: 85%;
    font-size: 1.5em;
    list-style: none;
    padding-left: 20px;

    li {
        padding-bottom: 40px;
        position: relative
    }

    li>span {
        margin-left: -11px;
    }

    li:last-child {
        padding-bottom: 0;
    }

    li>span:before {
      content: '';
      position: absolute;
      border-left: 2px solid rgba(255, 255, 255, 0.85);
      left: -10px;
      bottom: 0;
      height: 100%;
    }

    li>span:after {
      content: '';
      position: absolute;
      border-left: 2px solid rgba(255, 255, 255, 0.85);
      left: -10px;
      bottom: 0;
      height: 100%;
    }

    li:last-child span: before{
     content: none;
    }
`;

const cssProp = css`
    font-family: Computer Modern;
`;

const cssHint = css`
    position: relative;

    color: #1e1f22;
    background-color: rgba(30, 31, 34, 1);
    border-radius: 5px;

    :hover {
        color: inherit;
        background-color: rgba(30, 31, 34, 0.25);
        transition: all 0.15s;
    }
`;