export function createNumberGenerator() {
    const fn = function generator() {
        const num = generator.counter;
        generator.counter = num + 1;

        return num;
    };

    fn.counter = 0;

    return fn;
}