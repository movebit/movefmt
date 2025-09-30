// #issue34
module aptos_std::string_utils {
    use std::string::String;

    /// The number of values in the list does not match the number of "{}" in the format string.
    const EARGS_MISMATCH: u64 = 1;
    /// The format string is not valid.
    const EINVALID_FORMAT: u64 = 2;
    /// Formatting is not possible because the value contains delayed fields such as aggregators.
    const EUNABLE_TO_FORMAT_DELAYED_FIELD: u64 = 3;

    public fun to_string<T>(s: &T): String {
        native_format(s, false, false, true, false)
    }

    /// Format addresses as 64 zero-padded hexadecimals.
    public fun to_string_with_canonical_addresses<T>(s: &T): String {
        native_format(s, false, true, true, false)
    }

    /// Format emitting integers with types ie. 6u8 or 128u32.
    public fun to_string_with_integer_types<T>(s: &T): String {
        native_format(s, false, true, true, false)
    }

    /// Format vectors and structs with newlines and indentation.
    public fun debug_string<T>(s: &T): String {
        native_format(s, true, false, false, false)
    }

    // Helper struct to allow passing a generic heterogeneous list of values to native_format_list.
    struct Cons<T, N> has copy, drop, store {
        car: T,
        cdr: N
    }

    struct NIL has copy, drop, store {}

    // Create a pair of values.
    fun cons<T, N>(car: T, cdr: N): Cons<T, N> {
        Cons { car, cdr }
    }

    // Create a nil value.
    fun nil(): NIL {
        NIL {}
    }

    // Create a list of values.
    inline fun list1<T0>(a: T0): Cons<T0, NIL> {
        cons(a, nil())
    }

    inline fun list2<T0, T1>(a: T0, b: T1): Cons<T0, Cons<T1, NIL>> {
        cons(a, list1(b))
    }

    inline fun list3<T0, T1, T2>(a: T0, b: T1, c: T2): Cons<T0, Cons<T1, Cons<T2, NIL>>> {
        cons(a, list2(b, c))
    }

    inline fun list4<T0, T1, T2, T3>(a: T0, b: T1, c: T2, d: T3)
        : Cons<T0, Cons<T1, Cons<T2, Cons<T3, NIL>>>> {
        cons(a, list3(b, c, d))
    }
}
