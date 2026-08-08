fn main() {
    let arg = std::env::args().nth(1).expect("usage: dump <request-json>");
    println!("{}", canonical_lib::dispatch(&arg));
}
