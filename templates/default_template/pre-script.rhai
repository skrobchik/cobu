let bin_template = `use crads::Scanner;

fn solve(input: &mut Scanner) {
    let a: i32 = input.next();
    let b: i32 = input.next();
    println!("{}", a + b);
}

fn main() {
    let mut input = Scanner::default();
    let t: u32 = input.next();
    for i in 1..=t {
        #[cfg(debug_assertions)]
        println!("Test Case {i}");

        solve(&mut input);
    }
}

`;
let last_letter = variable::prompt("Last problem letter:", "F", "[a-zA-Z]");
last_letter = to_lower(last_letter);
for x in "abcdefghijklmnopqrstuvwxyz" {
    file::write("src/bin/" + x + ".rs", bin_template);
    if x == last_letter { break; }
}
file::delete("src/bin/.gitkeep")
