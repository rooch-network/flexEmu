use std::env::args;

const CS: [u64; 4] = [1, 2, 3, 4];

// Calculate `x^3 + 2 * x^2 + 3 * x + 4`.
// e.g. x=1, exp = 1+2+3+4 = 10
fn main() {
    let mut args = args();
    let _ = args.next().unwrap(); // first is binary name
    let input: u64 = args.next().unwrap().parse().unwrap();
    let output: u64 = args.next().unwrap().parse().unwrap();
    let mut xs: Vec<_> = (0..=3).map(|p| input.pow(p)).collect();
    xs.reverse();
    let y: u64 = xs.iter().zip(CS.iter()).map(|(x, c)| (*c) * (*x)).sum();
    assert_eq!(y, output, "expect {}, but got {}", output, y);
}
