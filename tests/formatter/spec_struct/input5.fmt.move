/// test_point: Struct ability written on multiple lines

/// The `ASCII` module defines basic string and char newtypes in Move that verify
/// that characters are valid ASCII, and that strings consist of only valid ASCII characters.
module std::ascii {
    use std::vector;
    use std::option::{Self, Option};

    /// An ASCII character.
    struct Char has copy, /*comment*/
    /*comment*/ drop,
    // comment
    store {
        // comment
        byte: u8
    }

    spec Char {
        // comment
        invariant is_valid_char(byte); //comment
    }
}
