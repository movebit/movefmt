address 0x2 {
module Token {
    struct Coin<AssetType: copy + drop> has store {
        type: AssetType,
        value: u64
    }

    // control the minting/creation in the defining module of `ATy`
    public fun create<ATy: copy + drop + store>(type: ATy, value: u64): Coin<ATy> {
        Coin { type, value }
    }

    public fun value<ATy: copy + drop + store>(coin: &Coin<ATy>): u64 {
        coin.value
    }
}
}

address 0xB055 {
module OneToOneMarket {
    use std::signer;
    use 0x2::Token;

    struct Pool<AssetType: copy + drop> has key {
        coin: Token::Coin<AssetType>
    }

    struct DepositRecord<phantom InputAsset: copy + drop, phantom OutputAsset: copy + drop> has key {
        record: u64
    }

    struct BorrowRecord<phantom InputAsset: copy + drop, phantom OutputAsset: copy + drop> has key {
        record: u64
    }

    struct Price<phantom InputAsset: copy + drop, phantom OutputAsset: copy + drop> has key {
        price: u64
    }

    fun max_borrow_amount<In: copy + drop + store, Out: copy + drop + store>(
        account: &signer
    ): u64 acquires Price, Pool, DepositRecord, BorrowRecord {
        let input_deposited = deposited_amount<In, Out>(account);
        let output_deposited = borrowed_amount<In, Out>(account);

        let input_into_output =
            input_deposited * borrow_global<Price<In, Out>>(@0xB055).price;
        let max_output =
            if (input_into_output < output_deposited) 0
            else (input_into_output - output_deposited);
        let available_output = {
            let pool = borrow_global<Pool<Out>>(@0xB055);
            Token::value(&pool.coin)
        };
        if (max_output < available_output) max_output
        else available_output

    }

    fun borrowed_amount<In: copy + drop + store, Out: copy + drop + store>(
        account: &signer
    ): u64 acquires BorrowRecord {
        let sender = signer::address_of(account);
        if (!exists<BorrowRecord<In, Out>>(sender)) return 0;
        borrow_global<BorrowRecord<In, Out>>(sender).record
    }
}
}

address 0x70DD {
module ToddNickels {
    use 0x2::Token;
    use std::signer;

    struct T has copy, drop, store {}

    struct Wallet has key {
        nickels: Token::Coin<T>
    }

    public fun init(account: &signer) {
        assert!(signer::address_of(account) == @0x70DD, 42);
        move_to(
            account,
            Wallet { nickels: Token::create(T {}, 0) }
        )
    }
}
}
