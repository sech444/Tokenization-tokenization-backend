use serde::{Serialize, Deserialize};
use sqlx::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "listing_type", rename_all = "lowercase")] 
pub enum ListingType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")] 
pub enum OrderStatus {
    Pending,
    Completed,
    Cancelled,
}
