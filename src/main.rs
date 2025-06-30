mod compiler;

fn main() {
    // let x = compiler::add2(1, 2);
    let x = compiler::parser::add(1, 2);
    println!("Result of add2: {}", x);
}
