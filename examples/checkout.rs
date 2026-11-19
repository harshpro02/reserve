use reserve::reservations::Inventory;

fn main() {
    let mut inventory = Inventory::new(5);
    let alice = inventory.reserve(3, 100, 60).unwrap();
    let bob = inventory.reserve(2, 100, 60).unwrap();
    println!("Alice's hold: {alice:?}");
    println!("Bob's hold: {bob:?}");
    inventory.confirm(alice.id, 110).unwrap();
    inventory.cancel(bob.id, 115).unwrap();
    println!("Alice paid; Bob cancelled.");
    println!(
        "Physical stock: {}, available: {}",
        inventory.on_hand(),
        inventory.available()
    );
    let charlie = inventory.reserve(2, 120, 30).unwrap();
    inventory.expire(150);
    println!("Charlie timed out: {:?}", inventory.get(charlie.id));
    println!("Available again: {}", inventory.available());
}
