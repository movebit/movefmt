module Specs::StateLabels {
    struct Counter has key, store {
        value: u64
    }

    struct Resource has key, store {
        value: u64
    }

    fun create_counter(account: &signer, init_value: u64) {
        move_to(account, Counter { value: init_value })
    }

    spec create_counter {
        ensures publish<Counter>(
            signer::address_of(account), Counter { value: init_value }
        );
    }

    fun double_update(addr: address, v1: u64, v2: u64) acquires Counter {
        Counter[addr].value = v1;
        Counter[addr].value = v2;
    }

    spec double_update {
        ensures..S |~ update<Counter> (
            addr, update_field(old(Counter[addr]), value, v1)
        );
        ensures S..|~ update<Counter> (
            addr, update_field(S |~ Counter[addr], value, v2)
        );
    }

    fun double_remove(addr1: address, addr2: address): (Counter, Counter) acquires Counter {
        let r1 = remove_resource(addr1);
        let r2 = remove_resource(addr2);
        (r1, r2)
    }

    spec double_remove {
        ensures..S |~ result_1 == result_of<remove_resource> (addr1);
        ensures S..|~ result_2 == result_of<remove_resource> (addr2);
        aborts_if aborts_of<remove_resource> (addr2);
    }

    fun create_then_read(account: &signer, addr: address): u64 acquires Resource {
        move_to(account, Resource { value: 42 });
        read_resource(addr)
    }

    spec create_then_read {
        ensures S..|~ result == result_of<read_resource> (addr);
        ensures S |~ exists<Resource> (signer::address_of(account));
        ensures S |~ Resource[signer::address_of(account)] == Resource { value: 42 };
    }

    spec fun two_state_check(addr: address): bool {
        old(Counter[addr].value) < Counter[addr].value
    }

    fun three_calls(f: |u64| u64, g: |u64| u64, h: |u64| u64, x: u64): u64 {
        let a = f(x);
        let b = g(a);
        h(b)
    }

    spec three_calls {
        ensures..s1 |~ ensures_of<f> (x);
        ensures s1..s2 |~ ensures_of<g> (x);
        ensures s2..|~ ensures_of<h> (x);
    }
}
