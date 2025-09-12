-- Marketplace tables migration
-- Version: 003
-- Description: Add marketplace trading tables

-- Marketplace Listings
CREATE TABLE marketplace_listings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_id UUID NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL CHECK (quantity > 0),
    price_per_token NUMERIC(36, 18) NOT NULL CHECK (price_per_token > 0),
    listing_type TEXT NOT NULL CHECK (listing_type IN ('buy','sell')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active','filled','cancelled','expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Marketplace Orders
CREATE TABLE marketplace_orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    listing_id UUID NOT NULL REFERENCES marketplace_listings(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    seller_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_id UUID NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL,
    price_per_token NUMERIC(36, 18) NOT NULL,
    total_amount NUMERIC(36, 18) NOT NULL,
    status TEXT NOT NULL DEFAULT 'completed'
        CHECK (status IN ('pending','completed','cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- User Token Balances
CREATE TABLE user_token_balances (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_id UUID NOT NULL REFERENCES tokens(id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, token_id)
);

-- Indexes
CREATE INDEX idx_marketplace_listings_status ON marketplace_listings(status);
CREATE INDEX idx_marketplace_listings_token_id ON marketplace_listings(token_id);
CREATE INDEX idx_marketplace_orders_listing_id ON marketplace_orders(listing_id);
CREATE INDEX idx_marketplace_orders_buyer_id ON marketplace_orders(buyer_id);
CREATE INDEX idx_marketplace_orders_seller_id ON marketplace_orders(seller_id);
CREATE INDEX idx_user_token_balances_user_id ON user_token_balances(user_id);
CREATE INDEX idx_user_token_balances_token_id ON user_token_balances(token_id);
