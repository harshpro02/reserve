//! Reservation lifecycle for one product. Times are seconds supplied by the caller.
//! This keeps expiry tests deterministic without sleeping.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Held,
    Confirmed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    pub id: u64,
    pub quantity: u32,
    pub expires_at: u64,
    pub status: Status,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidQuantity,
    InvalidExpiry,
    InsufficientStock,
    NotFound,
    InvalidTransition,
    CapacityExceeded,
}

pub struct Inventory {
    on_hand: u32,
    next_id: u64,
    reservations: BTreeMap<u64, Reservation>,
}

impl Inventory {
    pub fn new(on_hand: u32) -> Self {
        Self {
            on_hand,
            next_id: 1,
            reservations: BTreeMap::new(),
        }
    }

    pub fn on_hand(&self) -> u32 {
        self.on_hand
    }

    pub fn held(&self) -> u32 {
        self.reservations
            .values()
            .filter(|r| r.status == Status::Held)
            .map(|r| r.quantity)
            .sum()
    }

    pub fn available(&self) -> u32 {
        self.on_hand - self.held()
    }

    pub fn get(&self, id: u64) -> Option<&Reservation> {
        self.reservations.get(&id)
    }

    pub fn expire(&mut self, now: u64) -> usize {
        let mut expired = 0;
        for reservation in self.reservations.values_mut() {
            if reservation.status == Status::Held && now >= reservation.expires_at {
                reservation.status = Status::Expired;
                expired += 1;
            }
        }
        expired
    }

    pub fn reserve(&mut self, quantity: u32, now: u64, ttl: u64) -> Result<Reservation, Error> {
        if quantity == 0 {
            return Err(Error::InvalidQuantity);
        }
        if ttl == 0 {
            return Err(Error::InvalidExpiry);
        }
        let expires_at = now.checked_add(ttl).ok_or(Error::InvalidExpiry)?;
        let next_id = self.next_id.checked_add(1).ok_or(Error::CapacityExceeded)?;
        self.expire(now);
        if quantity > self.available() {
            return Err(Error::InsufficientStock);
        }
        let reservation = Reservation {
            id: self.next_id,
            quantity,
            expires_at,
            status: Status::Held,
        };
        self.reservations
            .insert(reservation.id, reservation.clone());
        self.next_id = next_id;
        Ok(reservation)
    }

    pub fn confirm(&mut self, id: u64, now: u64) -> Result<Reservation, Error> {
        self.expire(now);
        let reservation = self.reservations.get_mut(&id).ok_or(Error::NotFound)?;
        match reservation.status {
            Status::Held => {
                reservation.status = Status::Confirmed;
                self.on_hand -= reservation.quantity;
            }
            Status::Confirmed => {} // Retrying confirmation must not sell twice.
            _ => return Err(Error::InvalidTransition),
        }
        Ok(reservation.clone())
    }

    pub fn cancel(&mut self, id: u64, now: u64) -> Result<Reservation, Error> {
        self.expire(now);
        let reservation = self.reservations.get_mut(&id).ok_or(Error::NotFound)?;
        match reservation.status {
            Status::Held => reservation.status = Status::Cancelled,
            Status::Cancelled => {} // A retry must not release stock twice.
            _ => return Err(Error::InvalidTransition),
        }
        Ok(reservation.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_sells_once_and_preserves_other_holds() {
        let mut stock = Inventory::new(5);
        let alice = stock.reserve(3, 100, 60).unwrap();
        let bob = stock.reserve(2, 100, 60).unwrap();
        assert_ne!(alice.id, bob.id);
        assert_eq!(stock.available(), 0);
        assert_eq!(
            stock.confirm(alice.id, 110).unwrap().status,
            Status::Confirmed
        );
        stock.confirm(alice.id, 111).unwrap();
        assert_eq!(
            (stock.on_hand(), stock.held(), stock.available()),
            (2, 2, 0)
        );
        assert_eq!(stock.cancel(alice.id, 111), Err(Error::InvalidTransition));
    }

    #[test]
    fn cancellation_releases_once_and_cannot_be_confirmed() {
        let mut stock = Inventory::new(5);
        let hold = stock.reserve(3, 100, 60).unwrap();
        stock.cancel(hold.id, 110).unwrap();
        stock.cancel(hold.id, 111).unwrap();
        assert_eq!((stock.on_hand(), stock.available()), (5, 5));
        assert_eq!(stock.confirm(hold.id, 112), Err(Error::InvalidTransition));
    }

    #[test]
    fn expiry_boundary_releases_stock_and_blocks_payment() {
        let mut stock = Inventory::new(5);
        let hold = stock.reserve(5, 100, 60).unwrap();
        assert_eq!(stock.expire(159), 0);
        assert_eq!(stock.available(), 0);
        assert_eq!(stock.confirm(hold.id, 160), Err(Error::InvalidTransition));
        assert_eq!(stock.get(hold.id).unwrap().status, Status::Expired);
        assert_eq!(stock.available(), 5);
        assert_eq!(stock.expire(161), 0);
    }

    #[test]
    fn new_reservations_reclaim_expired_holds() {
        let mut stock = Inventory::new(1);
        stock.reserve(1, 100, 10).unwrap();
        assert_eq!(stock.reserve(1, 109, 10), Err(Error::InsufficientStock));
        assert!(stock.reserve(1, 110, 10).is_ok());
    }

    #[test]
    fn invalid_inputs_do_not_allocate_ids_or_stock() {
        let mut stock = Inventory::new(5);
        assert_eq!(stock.reserve(0, 100, 60), Err(Error::InvalidQuantity));
        assert_eq!(stock.reserve(1, 100, 0), Err(Error::InvalidExpiry));
        assert_eq!(stock.reserve(1, u64::MAX, 1), Err(Error::InvalidExpiry));
        assert_eq!(stock.reserve(6, 100, 60), Err(Error::InsufficientStock));
        assert_eq!(stock.confirm(99, 100), Err(Error::NotFound));
        assert_eq!(stock.cancel(99, 100), Err(Error::NotFound));
        assert_eq!(stock.reserve(1, 100, 60).unwrap().id, 1);
    }
}
