module test {
    fun test1() {
        let y: u64 = 100;
        let x: u64 = 0;
        let z =
            if (y <= 10)
                // If y is less than or equal to 10
                y

            // xxxxxx
            else {
                y = 10; // Otherwise, set y to 10
            };
    }

    fun test2() {
        let y: u64 = 100;
        let x: u64 = 0;
        let z =
            if (y <= 10)
                // If y is less than or equal to 10
                y = y + 1
            // Increment y by 1
            else y = 10; // Otherwise, set y to 10
    }

    fun test3() {
        let y: u64 = 100;
        let x: u64 = 0;
        let z =
            if (y <= 10)
                // If y is less than or equal to 10
                y = y + 1
            // Increment y by 1
            else y = 10; // Otherwise, set y to 10
        y + x
    }

    fun test4() {
        let y: u64 = 100;
        let x: u64 = 0;
        if (y <= 10)
            // If y is less than or equal to 10
            // comment 222
            y = y + 1
        // Increment y by 1
        else y = 10; // Otherwise, set y to 10
        y + x
    }
}
