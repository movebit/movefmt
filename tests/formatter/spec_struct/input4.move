/// test_point:  Struct field has comments

/// The `ASCII` module defines basic string and char newtypes in Move that verify
/// that characters are valid ASCII, and that strings consist of only valid ASCII characters.
module std::ascii {
    use std::vector;
    use std::option::{Self, Option};

   /// An ASCII character.
   struct Char has copy, drop, store {
    // comment
       byte: u8,
   }


   spec Char {
       invariant is_valid_char(byte);
   }
}