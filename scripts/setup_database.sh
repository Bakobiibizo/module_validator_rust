#!/bin/bash

# Database configuration
DB_NAME="module_validator"
DB_USER="module_validator_user"
DB_PASSWORD="your_secure_password"

# Check if psql is installed
if ! command -v psql &> /dev/null
then
    echo "Error: psql could not be found. Please install PostgreSQL."
    exit 1
fi

# Create database and user
sudo -u postgres psql << EOF
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = '$DB_NAME') THEN
        CREATE DATABASE $DB_NAME;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '$DB_USER') THEN
        CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';
    END IF;
END
\$\$;
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF

# Create tables
PGPASSWORD=$DB_PASSWORD psql -h localhost -d $DB_NAME -U $DB_USER << EOF
CREATE TABLE IF NOT EXISTS modules (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    version VARCHAR(50),
    entry_point VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_modules_name ON modules(name);
ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_type VARCHAR(50);
EOF

echo "Database setup completed successfully."

# Generate config.yaml
cat > config.yaml << EOF
database_url: "postgres://$DB_USER:$DB_PASSWORD@localhost/$DB_NAME"
modules: []
EOF

echo "config.yaml file created successfully."

# Update .env file
cat > .env << EOF
DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@localhost/$DB_NAME"
EOF

echo ".env file updated successfully."

echo "Setup complete. Please ensure you have the required Rust dependencies installed."