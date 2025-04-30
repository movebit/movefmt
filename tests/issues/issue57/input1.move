module callbacks {
    struct Callbacks {
        callback_f: |address, bool, u64| Option<u64> has drop + copy,
    }
}