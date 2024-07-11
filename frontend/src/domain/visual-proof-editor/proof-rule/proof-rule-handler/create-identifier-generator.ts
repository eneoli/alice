import { reverse } from 'lodash';

export function createIdentifierGenerator() {
    const fn = function generator() {
        const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');
        const alphabetLength = alphabet.length;

        const numDigits = Math.floor(generator.idx / alphabetLength) + 1;

        let identifier = '';
        for (let i = 0; i < numDigits; i++) {
            identifier += alphabet[Math.floor(generator.idx / Math.pow(alphabetLength, i)) % alphabetLength];
        }

        generator.idx = generator.idx + 1;

        return reverse(identifier);
    }

    fn.idx = 0;
    fn.reset = () => fn.idx = 0;

    return fn;
}