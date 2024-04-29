module fractions {
    public type Fraction = Int;

    public func mul(i: Int, fr: Fraction): Int {
        (i * fr) / 2**64;
    };

    public func div(i: Int, fr: Fraction): Int {
        (i * 2**64) / fr;
    };

    public func fdiv(i: Int, j: Int): Fraction {
        (i * 2**64) / j;
    };
};