# 🔍 Alice - A Constructive Logic Proof Checker

![GitHub Workflow]
![Commit Activity]
![Last Commit]
![Top Language]
![Repo Size]

# About

Alice is a tool for verifying proofs in constructive logic, specifically designed to assist students in the study of constructive logic.

[Constructive logic](https://en.wikipedia.org/wiki/Intuitionistic_logic), also known as intuitionistic logic, is a form of logic that emphasizes the constructive nature of proofs.
Alice provides an interactive user interface in the notion of [natural deduction](https://en.wikipedia.org/wiki/Natural_deduction), where students can learn by doing — constructing proofs step by step, with immediate feedback and support.

In addition to the graphical user interface, Alice includes its own mini programming language that emphasizes the beauty of the [Curry-Howard Isomorphism](https://en.wikipedia.org/wiki/Curry%E2%80%93Howard_correspondence).
This isomorphism links logic and computation and is the foundation of modern proof assistants like [Lean](https://lean-lang.org/) and [Coq](https://coq.inria.fr/).
It can be experienced within Alice by seamlessly switching between the graphical user interface and a code editor while constructing proofs, helping to bridge the gap between theory and practical application in proof assistants.

Alice is served as a standalone web app, using Rust and WebAssembly for its backend and React for the frontend.

# [Try Alice now](https://alice.eneoli.de)

https://github.com/user-attachments/assets/5eb95673-c5f0-4552-8cd1-b251c0f4a62f

# Build Instructions

## Prerequisites

To build Alice, ensure that you have the following installed:

1. **Rust**: Install Rust from [rust-lang.org](https://www.rust-lang.org/).
2. **wasm-pack**: Install wasm-pack from [rustwasm.github.io](https://rustwasm.github.io/wasm-pack/).
3. **NodeJs** and **NPM**: Install NodeJs and NPM from [nodejs.org](https://nodejs.org/).

## Step 1: Build the WebAssembly Binary of the Backend

In the root directory of the project, run the following command to build the WebAssembly binary:

`wasm-pack build`

## Step 2: Build the Frontend

Navigate to the `frontend` directory. Make sure the required dependices are installed by running `npm install`. Then run

`npm run build`

This will bundle the frontend assets and the WebAssembly binary and the final files will be output to the `dist` folder in the project root directory. These can be served by any HTTP server.

# Tests

Alice has a series of autmatic tests. To run them, make sure you are in the project root and run:

`cargo test`

# License
Alice is licensed under the MIT License. See [LICENSE] for the full license text.

<!-----------------------{ Badges }--------------------------->

[GitHub Workflow]: https://github.com/eneoli/alice/actions/workflows/automatic_tests.yml/badge.svg
[GitHub Workflow]: https://github.com/eneoli/alice/actions/workflows/rust.yml/badge.svg
[Commit Activity]: https://img.shields.io/github/commit-activity/m/eneoli/alice/main
[Last Commit]: https://img.shields.io/github/last-commit/eneoli/alice
[Top Language]: https://img.shields.io/github/languages/top/eneoli/alice
[Repo Size]: https://img.shields.io/github/repo-size/eneoli/alice
[LICENSE]: https://github.com/eneoli/alice/blob/main/LICENSE
