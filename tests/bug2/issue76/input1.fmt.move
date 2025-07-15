module test {
    fun settle_single_trade<M: store + copy + drop>(
        self: &mut Market<M>,
        user_addr: address,
        price: Option<u64>,
        orig_size: u64,
        remaining_size: &mut u64,
        is_bid: bool,
        metadata: M,
        order_id: OrderIdType,
        client_order_id: Option<u64>,
        callbacks: &MarketClearinghouseCallbacks<M>,
        fill_sizes: &mut vector<u64>
    ): Option<OrderCancellationReason> {
        let result =
            self.order_book.get_single_match_for_taker(price, *remaining_size, is_bid);
        let (maker_order, maker_matched_size) = result.destroy_single_order_match();
        if (!self.config.allow_self_trade && maker_order.get_account() == user_addr) {
            self.cancel_maker_order_internal(
                &maker_order,
                maker_order.get_client_order_id(),
                maker_order.get_account(),
                maker_order.get_order_id(),
                std::string::utf8(b"Disallowed self trading"),
                maker_matched_size,
                callbacks
            );
            return option::none();
        };
        let fill_id = self.next_fill_id();
        let settle_result =
            callbacks.settle_trade(
                user_addr,
                order_id,
                maker_order.get_account(),
                maker_order.get_order_id(),
                fill_id,
                is_bid,
                maker_order.get_price(), // Order is always matched at the price of the maker
                maker_matched_size,
                metadata,
                maker_order.get_metadata_from_order()
            );

        let unsettled_maker_size = maker_matched_size;
        let settled_size = settle_result.get_settled_size();
        if (settled_size > 0) {
            *remaining_size -= settled_size;
            unsettled_maker_size -= settled_size;
            fill_sizes.push_back(settled_size);
            // Event for taker fill
            self.emit_event_for_order(
                order_id,
                client_order_id,
                user_addr,
                orig_size,
                *remaining_size,
                settled_size,
                option::some(maker_order.get_price()),
                is_bid,
                true,
                market_types::order_status_filled(),
                &std::string::utf8(b"")
            );
            // Event for maker fill
            self.emit_event_for_order(
                maker_order.get_order_id(),
                maker_order.get_client_order_id(),
                maker_order.get_account(),
                maker_order.get_orig_size(),
                maker_order.get_remaining_size() + unsettled_maker_size,
                settled_size,
                option::some(maker_order.get_price()),
                !is_bid,
                false,
                market_types::order_status_filled(),
                &std::string::utf8(b"")
            );
        };

        option::none()
    }
}
