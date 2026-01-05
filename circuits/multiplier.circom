// Multiplier circuit:
// Public inputs: a, ab (ab = a * b)
// Public inputs: a, ab (ab = a * b)
// Private witness: b
// Prove that a * b == ab

template Multiplier() {
    signal input a;
    signal input ab; // public
    signal private input b;

    signal product;
    product <== a * b;

    // enforce equality to public ab
    product === ab;
}

component main = Multiplier();
