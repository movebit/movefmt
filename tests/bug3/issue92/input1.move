#[test_only]
module aptos_market::event_utils {
    use std::option::Option;
    use aptos_framework::event;

    struct EventStore<phantom T: drop + copy + store> has drop {
        last_index: u64
    }

    public fun new_event_store<T: drop + copy + store>(): EventStore<T> {
        EventStore<T> { last_index: 0 }
    }

    public fun latest_emitted_events<T: drop + copy + store>(
        self: &mut EventStore<T>, limit: Option<u64>
    ): vector<T> {
        let events = event::emitted_events<T>();
        let end_index =
            if (limit.is_none()) {
                events.length()
            } else {
                let limit = limit.destroy_some();
                std::math64::min(self.last_index + limit, events.length())
            };

        let latest_events = events.slice(self.last_index, end_index);
        self.last_index = end_index;
        latest_events
    }
     #[event]
     struct SomeEvent {}     

     fun test_formatting() {
        let event_store = new_event_store<SomeEvent>();
        // This line of code will format an error result and cause the code to fail to compile
        let events = event_store.latest_emitted_events<SomeEvent>(option::none());  // <-- here
     }
}