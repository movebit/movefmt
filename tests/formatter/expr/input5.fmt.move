script {
    fun example(a: u64, b: u64, c: u64, d: u64, e: u64): u64 {
        let addition: u64 = a + b; // Addition operation
        let subtraction: u64 = a - b; // Subtraction operation
        let multiplication: u64 = a * b; // Multiplication operation
        let division: u64 = a / b; // Division operation
        let modulo: u64 = a % b; // Modulo operation
        let result: u64 = (addition * subtraction) / (multiplication + division) % modulo; //  Operator expression
        // Operator expression
        let intermediate1: u64 = ((a + b) * (c - d)) / (e + 1);
        let intermediate2: u64 = (a * (c + d)) - (b / (e - 2));
        let intermediate3: u64 = (a * {c + d}) - (b / {e - 2});
        result = (intermediate1 % intermediate2) + 7 * (a - d);

        let x: bool = true;
        x = !x;
        return result;
    }
}
