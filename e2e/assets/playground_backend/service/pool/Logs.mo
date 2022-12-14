module {
    public type Stats = {
        num_of_canisters: Nat;
        num_of_installs: Nat;
        cycles_used: Nat;
        error_out_of_capacity: Nat;
        error_total_wait_time: Nat;
        error_mismatch: Nat;
    };
    public let defaultStats : Stats = {
        num_of_canisters = 0;
        num_of_installs = 0;
        cycles_used = 0;
        error_out_of_capacity = 0;
        error_total_wait_time = 0;
        error_mismatch = 0;
    };
    public type EventType = {
        #getId : Nat;
        #outOfCapacity : Nat;
        #install;
        #mismatch;
    };
    public func updateStats(stats: Stats, event: EventType) : Stats {
        switch (event) {
        case (#getId(cycles)) { {
                 num_of_canisters = stats.num_of_canisters + 1;
                 cycles_used = stats.cycles_used + cycles;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity;
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch;
                                } };
        case (#outOfCapacity(time)) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity + 1;
                 error_total_wait_time = stats.error_total_wait_time + time;                 
                 error_mismatch = stats.error_mismatch;
                                  } };
        case (#install) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs + 1;
                 error_out_of_capacity = stats.error_out_of_capacity;
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch;
                          } };
        case (#mismatch) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity; 
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch + 1;
                           } };
        };
    };
}
