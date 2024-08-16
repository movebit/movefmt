address 0x2 {

module Token {
    struct Coin<AssetType: copy + drop> has store {
        type: AssetType,
        value: u64,
    }

}

}

address 0xB055 {
#[fmt::skip]
module OneToOneMarket {
    use std::signer;    use 0x2::Token;

    struct Pool<AssetType: copy + drop> has key {
        coin: Token::Coin<AssetType>,
    }

    fun borrowed_amount<In: copy + drop + store, Out: copy + drop + store>(account: &signer): u64
        acquires BorrowRecord
    {
        let sender = signer::address_of(account);
        if (!exists<BorrowRecord<In, Out>>(sender)) return 0;
        borrow_global<BorrowRecord<In, Out>>(sender).record
    }
}

}

address 0x70DD {

module ToddNickels {

    struct T has copy, drop, store {}


}

}
