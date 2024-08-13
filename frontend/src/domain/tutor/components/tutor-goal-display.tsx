import { css } from '@emotion/css';
import { print_prop, Prop } from 'alice';
import React from 'react';

interface TutorGoalDisplayGoal {
    prop: Prop,
    hint: string,
}

interface TutorGoalDisplayProps {
    goals: TutorGoalDisplayGoal[];
}

export function TutorGoalDisplay(props: TutorGoalDisplayProps) {
    const { goals } = props;

    return (
        <div className={cssGoalDisplayContainer}>
            <span className={cssTitle}>
                🚩 You have <span className={cssTitleGoalNumber}>{goals.length}</span> open goals:
            </span>

            <ul className={cssGoalList}>
                {
                    goals.map((goal, i) => (
                        <li key={i}>
                            <span>
                                ⊢ <span className={cssProp}>{print_prop(goal.prop)}</span>
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
    align-items: center;
`;

const cssTitle = css`
    color: white;
    font-size: 1.7em;
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

    :hover:after {
        opacity: 0.15;
        transition: all 0.1s;
    }

    :after {
        content: '';
        position: absolute;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: #1e1f22;
        border-radius: 5px;
    }
`;