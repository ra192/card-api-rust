use chrono::prelude::*;

pub enum CardType{
    VIRTUAL
}

pub struct Card {
    pub id: i32,
    pub card_type: CardType,
    pub created: DateTime<Local>,
    pub cust_id: i32,
    pub acc_id: i32
}