module aptos_experimental::market_types {
    use std::option::Option;
    use std::string::String;

    use aptos_experimental::order_book_types::OrderIdType;

    friend aptos_experimental::market;

    const EINVALID_ADDRESS: u64 = 1;
    const EINVALID_SETTLE_RESULT: u64 = 2;
    const EINVALID_TIME_IN_FORCE: u64 = 3;

    /// Order time in force
    enum TimeInForce has drop, copy, store {
        /// Good till cancelled order type
        GTC,
        /// Post Only order type - ensures that the order is not a taker order
        POST_ONLY,
        /// Immediate or Cancel order type - ensures that the order is a taker order. Try to match as much of the
        /// order as possible as taker order and cancel the rest.
        IOC
    }

    enum OrderStatus has drop, copy, store {
        /// Order has been accepted by the engine.
        OPEN,
        /// Order has been fully or partially filled.
        FILLED,
        /// Order has been cancelled by the user or engine.
        CANCELLED,
        /// Order has been rejected by the engine. Unlike cancelled orders, rejected
        /// orders are invalid orders. Rejection reasons:
        /// 1. Insufficient margin
        /// 2. Order is reduce_only but does not reduce
        REJECTED,
        SIZE_REDUCED
    }

    enum SettleTradeResult has drop {
        V1 {
            settled_size: u64,
            maker_cancellation_reason: Option<String>,
            taker_cancellation_reason: Option<String>,
        }
    }

    enum MarketClearinghouseCallbacks<M: store + copy + drop> has drop {
        V1 {
            /// settle_trade_f arguments: taker, taker_order_id, maker, maker_order_id, fill_id, is_taker_long, price, size
            settle_trade_f:  |address, OrderIdType, address, OrderIdType, u64, bool, u64, u64, M, M| SettleTradeResult has drop + copy,
            /// validate_settlement_update_f arguments: account, order_id, is_taker, is_long, price, size
            validate_order_placement_f: |address, OrderIdType, bool, bool, Option<u64>, u64, M| bool has drop + copy,
            /// place_maker_order_f arguments: account, order_id, is_bid, price, size, order_metadata
            place_maker_order_f: |address, OrderIdType, bool, u64, u64, M| has drop + copy,
            /// cleanup_order_f arguments: account, order_id, is_bid, remaining_size
            cleanup_order_f: |address, OrderIdType, bool, u64| has drop + copy,
            /// decrease_order_size_f arguments: account, order_id, is_bid, price, size
            decrease_order_size_f: |address, OrderIdType, bool, u64, u64| has drop + copy,
            normal_f: |bool, u64| has drop + copy,
        }
    }
}

