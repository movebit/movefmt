module test_spec_forall {
    spec GasCurve {
        /// Invariant 1: The minimum gas charge does not exceed the maximum gas charge.
        invariant min_gas <= max_gas;
        /// Invariant 2: The maximum gas charge is capped by MAX_U64 scaled down by the basis point denomination.
        invariant max_gas <= MAX_U64 / BASIS_POINT_DENOMINATION;
        /// Invariant 3: The x-coordinate increases monotonically and the y-coordinate increasing strictly monotonically,
        /// that is, the gas-curve is a monotonically increasing function.
        invariant (len(points) > 0 ==> points[0].x > 0)
            && (len(points) > 0 ==> points[len(points) - 1].x < BASIS_POINT_DENOMINATION)
            && (forall i in 0..len(points) - 1: (points[i].x < points[i + 1].x && points[i].y <= points[i + 1].y));
    }

}
