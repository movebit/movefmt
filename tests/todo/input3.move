module bcs_stream::bcs_stream {
    use std::error;
    use std::vector;
    use std::option::{Self, Option};
    use std::string::{Self, String};

    use aptos_std::from_bcs;

    /// The data does not fit the expected format.
    const EMALFORMED_DATA: u64 = 1;
    /// There are not enough bytes to deserialize for the given type.
    const EOUT_OF_BYTES: u64 = 2;

    struct BCSStream has drop {
        /// Byte buffer containing the serialized data.
        data: vector<u8>,
        /// Cursor indicating the current position in the byte buffer.
        cur: u64,
    }

    public fun deserialize_u256(stream: &mut BCSStream): u256 {
        let data = &stream.data;
        let cur = stream.cur;

        assert!(cur + 32 <= vector::length(data), error::out_of_range(EOUT_OF_BYTES));
        let res =
            (*vector::borrow(data, cur) as u256)
                |((*vector::borrow(data, cur + 1) as u256) << 8)
                |((*vector::borrow(data, cur + 2) as u256) << 16)
                |((*vector::borrow(data, cur + 3) as u256) << 24)
                |((*vector::borrow(data, cur + 4) as u256) << 32)
                |((*vector::borrow(data, cur + 5) as u256) << 40)
                |((*vector::borrow(data, cur + 6) as u256) << 48)
                |((*vector::borrow(data, cur + 7) as u256) << 56)
                |((*vector::borrow(data, cur + 8) as u256) << 64)
                |((*vector::borrow(data, cur + 9) as u256) << 72)
                |((*vector::borrow(data, cur + 10) as u256) << 80)
                |((*vector::borrow(data, cur + 11) as u256) << 88)
                |((*vector::borrow(data, cur + 12) as u256) << 96)
                |((*vector::borrow(data, cur + 13) as u256) << 104)
                |((*vector::borrow(data, cur + 14) as u256) << 112)
                |((*vector::borrow(data, cur + 15) as u256) << 120)
                |((*vector::borrow(data, cur + 16) as u256) << 128)
                |((*vector::borrow(data, cur + 17) as u256) << 136)
                |((*vector::borrow(data, cur + 18) as u256) << 144)
                |((*vector::borrow(data, cur + 19) as u256) << 152)
                |((*vector::borrow(data, cur + 20) as u256) << 160)
                |((*vector::borrow(data, cur + 21) as u256) << 168)
                |((*vector::borrow(data, cur + 22) as u256) << 176)
                |((*vector::borrow(data, cur + 23) as u256) << 184)
                |((*vector::borrow(data, cur + 24) as u256) << 192)
                |((*vector::borrow(data, cur + 25) as u256) << 200)
                |((*vector::borrow(data, cur + 26) as u256) << 208)
                |((*vector::borrow(data, cur + 27) as u256) << 216)
                |((*vector::borrow(data, cur + 28) as u256) << 224)
                |((*vector::borrow(data, cur + 29) as u256) << 232) |(
                (*vector::borrow(data, cur + 30) as u256) << 240
            ) |((*vector::borrow(data, cur + 31) as u256) << 248);

        stream.cur = stream.cur + 32;
        res
    }
}
