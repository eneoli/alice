pub struct IdentifierGenerator {
    idx: usize,
}

impl IdentifierGenerator {
    pub fn new() -> Self {
        Self { idx: 0 }
    }

    pub fn generate(&mut self) -> String {
        let alphabet = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<char>>();
        let alphabet_length = alphabet.len();
        let num_digits = f32::floor(self.idx as f32 / alphabet_length as f32) as usize + 1;

        let mut identifier = String::new();
        for i in 0..num_digits {
            identifier.push(
                alphabet[(f32::floor(
                    (self.idx as f32) / usize::pow(alphabet_length, i.try_into().unwrap()) as f32,
                ) as usize)
                    % alphabet_length],
            );
        }

        self.idx += 1;

        identifier.chars().rev().collect()
    }
}
