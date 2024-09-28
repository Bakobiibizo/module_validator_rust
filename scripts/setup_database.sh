#!/bin/bash

# Database configuration
read -p "Enter your database name: " DB_NAME
read -p "Enter your database user: " DB_USER
read -p "Enter your database password: " DB_PASSWORD

# Check if psql is installed
if ! command -v psql &> /dev/null
then
    sudo apt-get install -y postgresql postgresql-contrib

    sudo systemctl start postgresql

    sudo systemctl enable postgresql

    sudo systemctl status postgresql

    sudo -u postgres createuser --superuser $DB_USER

    sudo -u postgres createdb $DB_NAME

    sudo pg_ctlcluster 14 main start

    sudo nano /etc/postgresql/14/main/postgresql.conf
    listen_addresses = 'localhost'
    sudo systemctl restart postgresql

    # Replace the psql commands with:
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;"
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';"
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"
fi


# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE $DB_NAME;
CREATE USER $DB_USER WITH ENCRYPTED PASSWORD '$DB_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF

# Create tables
# Create tables
sudo -u postgres psql -d $DB_NAME << EOF
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
database_name: $DB_NAME
database_user: $DB_USER
database_password: $DB_PASSWORD
modules: []
EOF

echo "config.yaml file created successfully."

# Update .env file
cat > .env << EOF
DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@localhost/$DB_NAME"
DATABASE_NAME=$DB_NAME
DATABASE_USER=$DB_USER
DATABASE_PASSWORD=$DB_PASSWORD
EOF

echo ".env file updated successfully."

echo "Setup complete. Please ensure you have the required Rust dependencies installed."