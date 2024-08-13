import React from 'react';
import { TutorPropositionSolutionStatus } from './tutor-proposition-solution-status';
import { TutorGoalDisplay } from './tutor-goal-display';
import { parse_prop } from 'alice';

interface TutorProps {
}

export function Tutor(_prop: TutorProps) {
    return (
        <>
            <TutorPropositionSolutionStatus status={'unknown'} />
            <br />
            <hr style={{ borderColor: 'rgba(124, 178, 251, 0.25)' }} />
            <br />
            <TutorGoalDisplay goals={[
                {
                    prop: parse_prop('A & B'),
                    hint: 'Use rule &E1',
                }, {
                    prop: parse_prop('A & B'),
                    hint: 'Use rule &E1',
                }, {
                    prop: parse_prop('A & B'),
                    hint: 'Use rule &E1',
                },
            ]} />
        </>
    );
}
