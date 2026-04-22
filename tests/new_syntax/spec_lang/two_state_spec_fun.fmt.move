module Specs::TwoStateSpec {
    struct Counter has key, store {
        value: u64
    }

    struct Config has key, store {
        active: bool
    }

    struct Resource has key, store {
        value: u64
    }

    spec fun counter_increased(addr: address): bool {
        old(Counter[addr].value) < Counter[addr].value
    }

    fun increment_if_active(addr: address) acquires Counter, Config {
        if (Config[addr].active) {
            Counter[addr].value = Counter[addr].value + 1
        }
    }

    spec increment_if_active {
        pragma opaque;
        modifies Counter[addr];
        ensures Config[addr].active ==> counter_increased(addr);
    }

    fun increment_twice(addr: address) acquires Counter {
        Counter[addr].value = Counter[addr].value + 1;
        Counter[addr].value = Counter[addr].value + 1;
    }

    spec increment_twice {
        ensures..S |~ counter_increased(addr);
        ensures S..|~ counter_increased(addr);
    }

    spec fun counter_is_positive(addr: address): bool {
        Counter[addr].value > 0
    }

    spec fun counter_ok(addr: address): bool {
        counter_is_positive(addr)
    }

    fun apply(f: |address|, x: address) {
        f(x)
    }

    spec apply {
        pragma opaque;
        reads_of<f> Config;
        modifies_of<f> (a: address) Counter[a];
        ensures ensures_of<f> (x);
        aborts_if aborts_of<f> (x);
    }
}
