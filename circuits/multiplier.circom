pragma circom 2.0.0;

// Multiplier circuit: public inputs are a and a*b, witness includes b
// Prove that provided a and b multiply to the public output

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
