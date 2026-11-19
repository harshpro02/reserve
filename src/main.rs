use reserve::Stock;

fn main() {
    let mut stock = Stock::new("DEMO-TEE", 5);

    println!("Product: {}", stock.sku());
    println!("Starting available stock: {}", stock.available());
    println!("Alice requests 3: {:?}", stock.reserve(3));
    println!("Bob requests 2: {:?}", stock.reserve(2));
    println!("Items on the shelf: {}", stock.on_hand());
    println!("Items held: {}", stock.held());
    println!("Items still available: {}", stock.available());
}
