// Day 1: one product, one process, and a basic reservation rule.
// Database transactions and simultaneous buyers come in a later lesson.

pub mod reservations;

#[derive(Debug, PartialEq)]
pub enum ReserveError {
    InvalidQuantity,
    InsufficientStock,
}

pub struct Stock {
    sku: String,
    on_hand: u32,
    held: u32,
}

impl Stock {
    pub fn new(sku: &str, on_hand: u32) -> Self {
        Self {
            sku: sku.to_string(),
            on_hand,
            held: 0,
        }
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn on_hand(&self) -> u32 {
        self.on_hand
    }

    pub fn held(&self) -> u32 {
        self.held
    }

    pub fn available(&self) -> u32 {
        self.on_hand - self.held
    }

    pub fn reserve(&mut self, quantity: u32) -> Result<u32, ReserveError> {
        if quantity == 0 {
            return Err(ReserveError::InvalidQuantity);
        }

        if quantity > self.available() {
            return Err(ReserveError::InsufficientStock);
        }

        self.held += quantity;
        Ok(self.available())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserving_three_of_five_leaves_two_available() {
        let mut stock = Stock::new("TEE", 5);

        assert_eq!(stock.reserve(3), Ok(2));
        assert_eq!(stock.held(), 3);
        assert_eq!(stock.on_hand(), 5);
    }

    #[test]
    fn requesting_too_many_leaves_stock_unchanged() {
        let mut stock = Stock::new("TEE", 5);

        assert_eq!(stock.reserve(6), Err(ReserveError::InsufficientStock));
        assert_eq!(stock.available(), 5);
        assert_eq!(stock.held(), 0);
    }

    #[test]
    fn zero_is_not_a_valid_reservation() {
        let mut stock = Stock::new("TEE", 5);

        assert_eq!(stock.reserve(0), Err(ReserveError::InvalidQuantity));
        assert_eq!(stock.available(), 5);
        assert_eq!(stock.held(), 0);
    }

    #[test]
    fn later_requests_only_see_unreserved_stock() {
        let mut stock = Stock::new("TEE", 5);

        assert_eq!(stock.reserve(3), Ok(2));
        assert_eq!(stock.reserve(3), Err(ReserveError::InsufficientStock));
        assert_eq!(stock.reserve(2), Ok(0));
        assert_eq!(stock.reserve(1), Err(ReserveError::InsufficientStock));
        assert_eq!(stock.held(), 5);
    }
}
