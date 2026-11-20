CREATE TABLE IF NOT EXISTS reserve_stock (
 sku text PRIMARY KEY, name text NOT NULL, on_hand integer NOT NULL CHECK(on_hand>=0)
);
CREATE TABLE IF NOT EXISTS reserve_holds (
 id text PRIMARY KEY, sku text NOT NULL REFERENCES reserve_stock(sku),
 quantity integer NOT NULL CHECK(quantity>0),
 status text NOT NULL CHECK(status IN ('held','confirmed','cancelled','expired')),
 expires_at bigint NOT NULL, request_key text NOT NULL, ttl integer NOT NULL CHECK(ttl>0),
 UNIQUE(sku,request_key)
);
CREATE INDEX IF NOT EXISTS reserve_active ON reserve_holds(sku,expires_at) WHERE status='held';
INSERT INTO reserve_stock VALUES('TEE','Everyday tee',20),('PACK','Trail backpack',8),('CAP','Weekend cap',12) ON CONFLICT DO NOTHING;
