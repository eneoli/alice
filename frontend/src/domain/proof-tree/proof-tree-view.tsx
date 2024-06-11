import React, { MouseEvent, ReactNode, useEffect, useRef, useState } from 'react';
import { css } from "@emotion/css";

interface ProofTreeProps {
    children: ReactNode;
}

export function ProofTreeView({ children }: ProofTreeProps) {
    const state = useRef({ x: 0, y: 0, mouseX: 0, mouseY: 0, scale: 1, isDragging: false });

    const container = useRef<HTMLDivElement>(null);
    const element = useRef<HTMLDivElement>(null);

    const refresh = () => {
        if (element.current) {
            element.current.style.transform = `translate3d(${state.current.x}px, ${state.current.y}px, 0) scale(${state.current.scale})`;
        }
    }

    const onWheel = (e: WheelEvent) => {
        const scaleDelta = e.deltaY * -0.001;
        const scale = Math.max(0.25, Math.min(state.current.scale + scaleDelta, 5.0));

        state.current = {
            ...state.current,
            scale: scale,
        };

        refresh();

        e.preventDefault();
        e.stopPropagation();
    };

    // register Wheel Event
    useEffect(() => {
        container.current?.addEventListener('wheel', onWheel, { passive: false });

        return () => container.current?.removeEventListener('wheel', onWheel);
    }, [container.current]);



    const handleMouseMove = (e: MouseEvent<HTMLDivElement>) => {
        const x = e.pageX;
        const y = e.pageY;

        const { mouseX, mouseY, scale } = state.current;

        const containerWidth = container.current?.offsetWidth || 0;
        const containerHeight = container.current?.offsetHeight || 0;

        const elementWidth = element.current?.offsetWidth || 0 * 1 / scale;
        const elementHeight = element.current?.offsetHeight || 0 * 1 / scale;

        if (state.current?.isDragging) {
            state.current = {
                ...state.current,
                x: Math.max(0, Math.min(state.current.x + (x - mouseX), containerWidth - elementWidth * scale)),
                y: Math.max(0, Math.min(state.current.y + (y - mouseY), containerHeight - elementHeight * scale)),
            };

            refresh();
        }

        state.current.mouseX = x;
        state.current.mouseY = y;
    };

    const onObjectMousePress = (e: MouseEvent<HTMLDivElement>) => {
        state.current.mouseX = (e.pageX);
        state.current.mouseY = (e.pageY);
        state.current.isDragging = true;
    };

    const onObjectMouseRelease = () => {
        state.current.isDragging = false;
    };

    return (
        <div ref={container}
            style={{ width: '500px', height: '500px', margin: 'auto', padding: '50px' }}
            className={cssContainer}
            onMouseMove={handleMouseMove}
            onMouseUp={onObjectMouseRelease}>
            <div ref={element}
                style={{ transformOrigin: 'center' }}
                className={cssProofTreeContainer}
                onMouseDown={onObjectMousePress}
                onMouseUp={onObjectMouseRelease}>
                {children}
            </div>
        </div>
    );
}

const cssContainer = css`
    position: relative;
    user-select: none;
    border: 1px solid grey;
`;

const cssProofTreeContainer = css`
    cursor: default;
    position: absolute;
`;