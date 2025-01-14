pub mod journal;

// Partitions enum with explicit discriminant values
pub enum Partitions {
    System = 1,
    Market = 2,
}

// Sections enum with a structure that supports hardcoded mapping
pub enum SystemSections {
    Organization = 1,
}

pub enum MarketSections {
    Token = 1,
    OrderBook = 2,
}

