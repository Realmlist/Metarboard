import sqlite3

def initialize_database():
    conn = sqlite3.connect("config.db")
    cursor = conn.cursor()

    # Create the settings table
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    )
    """)

    # Insert default settings
    default_settings = {
        "weather_type": "METAR",
        "station": "EHGR",
        "interval": "5"
    }

    for key, value in default_settings.items():
        cursor.execute("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)", (key, value))

    conn.commit()
    conn.close()

if __name__ == "__main__":
    initialize_database()
    print("Database initialized with default settings.")