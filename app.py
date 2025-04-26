from flask import Flask, render_template, request, redirect, url_for
import sqlite3
import signal
import sys
import logging
from datetime import datetime
import httpx
import vesta
import threading
import os

app = Flask(__name__)

# Database functions
def get_all_settings():
    conn = sqlite3.connect("config.db")
    cursor = conn.cursor()
    cursor.execute("SELECT key, value FROM settings")
    settings = cursor.fetchall()
    conn.close()
    return settings

def update_setting(key, value):
    conn = sqlite3.connect("config.db")
    cursor = conn.cursor()
    cursor.execute("UPDATE settings SET value = ? WHERE key = ?", (value, key))
    conn.commit()
    conn.close()

def get_setting(key):
    conn = sqlite3.connect("config.db")
    cursor = conn.cursor()
    cursor.execute("SELECT value FROM settings WHERE key = ?", (key,))
    result = cursor.fetchone()
    conn.close()
    if result:
        return result[0]
    raise KeyError(f"Setting '{key}' not found in the database.")

# Function to initialize the database if it doesn't exist
def initialize_database():
    if not os.path.exists("config.db"):
        logging.info("Database not found. Initializing...")
        os.system("python init_db.py")

# Flask routes
@app.route('/')
def index():
    settings = get_all_settings()
    return render_template('index.html', settings=settings)

# Function to handle the update form submission
@app.route('/update', methods=['POST'])
def update():
    # Update keys
    for key, value in request.form.items():
        update_setting(key, value)
    # Restart the main loop
    restart_main_loop()
    return redirect(url_for('index'))

main_loop_thread = None
stop_main_loop = threading.Event()

# Function to restart the main loop
def restart_main_loop():
    global main_loop_thread, stop_main_loop
    if main_loop_thread and main_loop_thread.is_alive():
        stop_main_loop.set()
        main_loop_thread.join()
    stop_main_loop.clear()
    main_loop_thread = threading.Thread(target=main_loop)
    main_loop_thread.start()

# Define a class to hold settings
class Settings:
    @property
    def weather_type(self):
        return get_setting("weather_type")

    @property
    def station(self):
        return get_setting("station")

    @property
    def interval(self):
        return int(get_setting("interval"))

    @property
    def api_key(self):
        return os.getenv("API_KEY", "")  # Read API_KEY from environment variable

settings = Settings()

# Configure logging (saves to file + prints to console)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[
        #logging.FileHandler("metarboard.log"), # Uncomment to save to file
        logging.StreamHandler(sys.stdout)
    ]
)

# Graceful shutdown handler
def shutdown_handler(_signum, _frame):
    logging.info("Shutting down gracefully...")
    sys.exit(0)

# Register SIGINT (Ctrl+C) and SIGTERM (kill) handlers
signal.signal(signal.SIGINT, shutdown_handler)
signal.signal(signal.SIGTERM, shutdown_handler)

# Function to fetch raw METAR/TAF data
def get_api_data():
    """Fetch raw METAR or TAF data from the Aviation Weather API"""
    url = f"https://aviationweather.gov/api/data/{settings.weather_type.lower()}?ids={settings.station}&format=raw"
    response = httpx.get(url)
    return response.text

# Function to get the current time
def get_time():
    now = datetime.now()
    return now.strftime("%H%M")

# Function to parse the METAR/TAF data for military color codes
def parse_mil_color(api_data):
    """Parse METAR/TAF data for military color codes"""
    color_map = {
        "RED": "{63}",
        "AMB": "{64}",
        "YLO": "{65}",
        "GRN": "{66}",
        "WHT": "{69}",
        "BLU": "{67}"
    }
    for key, value in color_map.items():
        if key in api_data:
            return value
    return " " # Return blank if no color found

# Function to get the VFR color code based on visibility and ceiling
def get_vfr_color_code(api_data):
    """Get VFR color code based on visibility and ceiling"""

    # Initialize default values
    ceiling_ft = 9999
    visibility_miles = 10.0

    parts = api_data.split()

    # Parse visibility and ceiling from the METAR/TAF data
    for i, part in enumerate(parts):
        if 'SM' in part:
            try:
                vis_str = part.split('SM')[0]
                if '/' in vis_str:
                    num, denom = vis_str.split('/')
                    visibility_miles = float(num) / float(denom)
                else:
                    visibility_miles = float(vis_str)
                break
            except (ValueError, IndexError):
                continue
        if (len(part) == 4 and part.isdigit() and
            not any(x in parts[max(0,i-1):i+1] for x in ['KT', 'MPS', 'KMH', 'Z'])):
            try:
                visibility_miles = int(part) / 1609.34
                break
            except ValueError:
                continue

    # Parse cloud cover and ceiling from the METAR/TAF data
    cloud_cover = None
    for part in parts:
        if part.startswith(('FEW', 'SCT', 'BKN', 'OVC', 'OVX', 'VV')):
            cloud_cover = part[:3]
            try:
                ceiling_ft = int(part[3:6]) * 100
                break
            except (ValueError, IndexError):
                continue
        elif part in ['NSC', 'CLR', 'SKC', 'CAVOK']:
            ceiling_ft = 9999

    # Determine color based on visibility and ceiling
    if ceiling_ft < 500 or visibility_miles < 1:
        color = '{68}'
    elif ceiling_ft < 1000 or visibility_miles < 3:
        color = '{63}'
    elif ceiling_ft <= 3000 or visibility_miles <= 5:
        color = '{67}'
    else:
        color = '{66}'

    # Determine name based on visibility
    if visibility_miles < 1:
        name = "LIFR"
    elif visibility_miles < 3:
        name = "IFR "
    elif visibility_miles < 5:
        name = "MVFR"
    else:
        name = "VFR "
    
    # Determine colour pattern based on cloud cover
    if cloud_cover:
        white = '{69}'
        match cloud_cover:
            case "FEW":
                return  name + white*3 + color
            case "SCT":
                return  name + white*2 + color*2
            case "BKN":
                return  name + white + color*3
            case "OVC" | "OVX" | "VV":
                return color * 4
    return name + white*3 + color

# Function to send data to Vestaboard
def send_to_vesta():
    """Send METAR or TAF data to Vestaboard"""
    # METAR Vestaboard layout
    # MET VFR0000 MIL0 JT0000
    # METAR DUMP

    # TAF Vestaboard layout
    # 0000(JT) TAF DUMP

    # Format the data for Vestaboard
    api_data = get_api_data()
    if settings.weather_type.upper() == "TAF":
        formatted = f"{get_time()} {api_data.replace('\n', '')}"
    else:
        formatted = f"MET {get_vfr_color_code(api_data)}MIL{parse_mil_color(api_data)}JT{get_time()}\n{api_data.replace('\n', '')}"

    print(f"Formatted data: {formatted}")
    # Send to Vestaboard
    rw_client = vesta.ReadWriteClient(settings.api_key)
    encoded_text = vesta.encode_text(formatted.replace("\\", "/"))
    assert rw_client.write_message(encoded_text)
    return formatted

# Main loop to periodically fetch and send data
def main_loop():
    logging.info("Starting Metarboard loop (%d-minute intervals)", settings.interval)
    while not stop_main_loop.is_set():
        try:
            result = send_to_vesta()
            logging.info("Metarboard update successful: %s", result)
            stop_main_loop.wait(settings.interval * 60)
        except Exception as e:
            logging.error("Metarboard update failed: %s", str(e), exc_info=True)
            stop_main_loop.wait(60)

if __name__ == '__main__':
    initialize_database()
    flask_thread = threading.Thread(target=lambda: app.run(host='0.0.0.0', debug=True, use_reloader=False))
    flask_thread.start()
    restart_main_loop()