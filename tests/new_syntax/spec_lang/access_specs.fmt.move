module Specs::AccessSpecifiers {
    struct Counter has key {
        value: u64
    }

    struct Config has key {
        active: bool
    }

    struct Data has key {
        value: u64
    }

    fun apply_reads(f: |address| u64, x: address): u64 {
        f(x)
    }

    spec apply_reads {
        pragma opaque;
        reads_of<f> Config;
        ensures result == result_of<f> (x);
    }

    fun apply_writes(f: |address| u64, x: address): u64 {
        f(x)
    }

    spec apply_writes {
        pragma opaque;
        modifies Data[x];
        modifies_of<f> (a: address) Data[a];
        ensures ensures_of<f> (x, result);
        aborts_if aborts_of<f> (x);
    }

    fun apply_mixed(f: |address| u64, x: address): u64 {
        f(x)
    }

    spec apply_mixed {
        pragma opaque;
        modifies Data[x];
        reads_of<f> Config;
        modifies_of<f> (a: address) Data[a];
        ensures ensures_of<f> (x, result);
        aborts_if aborts_of<f> (x);
    }

    fun my_fun(x: address): u64 acquires Data {
        Data[x].value
    }

    spec my_fun {
        reads R, S;
        modifies R[addr];
    }
}
