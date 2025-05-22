
## command parameter 
### 1.--stdin or -i
> `movefmt` receive source code text from stdin


eg:
```bash
% echo "module   test {               }" | movefmt -i 
2025-05-22T14:02:50.526424Z  WARN movefmt: 
The formatted result of the Move code read from stdin is as follows:
--------------------------------------------------------------------
module test {}

% 
% echo "module   test {               }" | movefmt --stdin -v
Spent 0.000 secs in the parsing phase, and 0.000 secs in the formatting phase
2025-05-22T14:03:44.985581Z  WARN movefmt: 
The formatted result of the Move code read from stdin is as follows:
--------------------------------------------------------------------
module test {}

% 
% cat tests/formatter/expr/input1.move | movefmt -i -v
Spent 0.005 secs in the parsing phase, and 0.001 secs in the formatting phase
2025-05-22T14:04:38.110167Z  WARN movefmt: 
The formatted result of the Move code read from stdin is as follows:
--------------------------------------------------------------------
script {
    fun main() {
        let y: u64 = 100;
        let x: u64 = 0; // Define an unsigned 64-bit integer variable y and assign it a value of 100
        let z =
            /* you can comment everywhere */ if (y <= 10) { // If y is less than or equal to 10
                y = y + 1; // Increment y by 1
            } else {
                y = /* you can comment everywhere */ 10; // Otherwise, set y to 10
            };
        let z2 =
            if (x = 0) { // If x equals 0
                x = x + 2; // x increases by 2
            } else {
                x = 10; // Otherwise, set y to 10
            };
        /*
        aa
        bb
        */
    }
}

%

```

