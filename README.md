# COBU: Competitive Bundler

Workspace for COBU and other projects for competitive programming in Rust.

COBU is a bundler that is designed for online judges like Codeforces.

It will:
  - Bundle your binary and libraries into a single source file
  - Remove any dead code like data structures you didn't use in your solution.
  - Run rustfmt.
  - Save the resulting code into `./dist` ready for submission

## Dependencies

You will need cargo-generate to use the contest templates.

```
cargo install cargo-generate
```

## Usage

Clone the repository.

```
git clone https://github.com/skrobchik/cobu.git
cd cobu
```

The `contests` directory is where all the code for your contests will go. All crates
that are inside `contests` are included in the Cargo workspace. So, the libraries will
only get compiled once for all contests.

Go into the `contests` directory and create a new contest from the template.

```
cd contests
cargo generate ..
```

Cargo generate will prompt you about the contest name and the letter of the last problem.
Contests on Codeforces usually correspond to a letter. You might have problems: A, B, C, D, E, F, G.
In this case the last problem letter is G.

```
⚠️   Favorite `..` not found in config, using it as a local path: ..
✔ 🤷   Which sub-template should be expanded? · contest-template
🤷   Project Name: codeforces1008
🔧   Destination: /home/robert/GitProjects/cobu/contests/codeforces1008 ...
🔧   project-name: codeforces1008 ...
🔧   Generating template ...
🤷   Last problem letter:: G
🔧   Moving generated files into: `/home/robert/GitProjects/cobu/contests/codeforces1008`...
🔧   Initializing a fresh Git repository
✨   Done! New project created /home/robert/GitProjects/cobu/contests/codeforces1008
```

In the generated template, the source files for the problems are in `src/bin`. You will
have a binary for each letter. You will also notice a `build.rs` script. Whenever you run
`cargo build`, this script will run COBU and place the files in the `dist` directory.

That's it! You should be ready to submit the source files directly from the `dist`
directory. Oh, actually... there's one more thing, you have to solve the problems
before submitting! So good luck on your contest, and hopefully it's all AC and no WA.

## Notes and Limitations

Feel free to fork this repository and customize the libraries to your liking. The only
library that currently gets used by the bundler is in this same workspace under the `crads` directory. COBU should support bundling multiple libraries, but I haven't tested it out.

The method for bundling is currently very naive. It creates a module with the same name
as the crate and copies and pastes all the library source files into it. Instead of trying to smartly decide which modules to copy or not, I decided to let the compiler
dead code elimination diagnostics figure it out and prune out all the code that doesn't
get used. The last time I tested, there were some issues with the dead code diagnostic and traits. That is why if you read some of the code in `crads` you will find the occasional `[allow(dead_code)]`.
