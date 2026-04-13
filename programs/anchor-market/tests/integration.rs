pub mod common;

mod cases {
    pub mod test_claim;
    pub mod test_claim_before_settled;
    pub mod test_initialize;
    pub mod test_merge;
    pub mod test_merge_after_settled;
    pub mod test_merge_invalid;
    pub mod test_multiple_markets;
    pub mod test_multiple_users;
    pub mod test_set_winner;
    pub mod test_set_winner_twice;
    pub mod test_set_winner_unauthorized;
    pub mod test_split;
    pub mod test_split_after_settled;
    pub mod test_split_insufficient_balance;
    pub mod test_split_invalid;
}
