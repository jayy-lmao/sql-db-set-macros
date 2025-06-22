-- Create user_status enum
CREATE TYPE user_status AS ENUM ('verified', 'unverified');

-- Create users table
CREATE TABLE users (
    name text NOT NULL,
    id text NOT NULL,
    email text NOT NULL,
    details text,
    status user_status NOT NULL
);
