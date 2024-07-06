// TODO: make inifnitly generateable

const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');

let idx = 0;

export function generateIdentifier() {
    const selectedIdx = idx;
    idx = idx + 1;

    return alphabet[selectedIdx];
}