import { useEffect, useState } from 'react';

type KeyHandler = (event: KeyboardEvent) => void;

export const useKeyDownEvent = createKeyHook('keydown');
export const useKeyUpEvent = createKeyHook('keyup');

export const useKeyPressed = (key: string) => {
    const [pressed, setPressed] = useState(false);

    useKeyDownEvent(() => setPressed(true), [key]);
    useKeyUpEvent(() => setPressed(false), [key]);

    return pressed;
};

function createKeyHook(eventName: 'keyup' | 'keydown') {
    return (onKey: KeyHandler, keyCodesListenedFor?: string[]) => {
        useEffect(() => {
            const handleKey = (event: KeyboardEvent) => {
                if (keyCodesListenedFor) {
                    if (keyCodesListenedFor.includes(event.key)) {
                        onKey(event);
                    }
                } else {
                    onKey(event);
                }
            };
            window.addEventListener(eventName, handleKey);

            return () => {
                window.removeEventListener(eventName, handleKey);
            };
        }, [onKey, keyCodesListenedFor]);
    }
}