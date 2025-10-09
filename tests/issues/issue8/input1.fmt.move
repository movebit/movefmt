// first module
module 0xc0ffee::m {
    fun test_return_with_nest() {
        event::emit(
            GameCreatedEvent {
                creator_address: player1_addr,
                opponent_address: player2_addr,
                object_address: object::object_address(&obj)
            }
        );

        event::emit(
            &GameCreatedEvent {
                creator_address: player1_addr,
                opponent_address: player2_addr,
                object_address: object::object_address(&obj)
            }
        );

        assert!(
            event::was_event_emitted(
                &MyEvent {
                    seq: 1111111111111111111111111111111111111111111111111100000000000011111100001,
                    field111111111111111111111111111111111111111111111111111111111: Field {
                        field: false
                    },
                    bytes: vector[]
                }
            ),
            111111111111111111111111111111111111111111111111111111111
        );
    }
}
