from config import settings
import httpx
from datetime import datetime
import signal
import sys
import logging
import time
import vesta

# Configure logging (saves to file + prints to console)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[
        logging.FileHandler("vesta_sender.log"),
        logging.StreamHandler(sys.stdout)
    ]
)

# Graceful shutdown handler
def shutdown_handler(signum, frame):
    logging.info("Shutting down gracefully...")
    sys.exit(0)

# Register SIGINT (Ctrl+C) and SIGTERM (kill) handlers
signal.signal(signal.SIGINT, shutdown_handler)
signal.signal(signal.SIGTERM, shutdown_handler)

def get_api_data() -> str:
    url = f"https://aviationweather.gov/api/data/{settings.weather_type.lower()}?ids={settings.station}&format=raw"
    response = httpx.get(url)
    return response.text

api_data = get_api_data()

def get_julia_time() -> str:
    now = datetime.now()
    julia_time = now.strftime("%H%M")
    return julia_time

def parse_mil_color() -> str:
    color_map = {
        "RED": "{63}",
        "AMB": "{64}",
        "YLO": "{65}",
        "GRN": "{66}",
        "WHT": "{71}",
        "BLU": "{67}"
    }
    
    for key, value in color_map.items():
        if key in api_data:
            return value
    
    return " "

def get_vfr_color_code():
    """
    Determine VFR color code from METAR string.
    
    Returns:
        str: Color code ('VFR', 'MVFR', 'IFR', or 'LIFR')
    """
    ceiling_ft = 9999  # Default high ceiling (VFR)
    visibility_miles = 10.0  # Default good visibility (VFR)
    
    parts = api_data.split()
    
    # Find visibility
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
    
    # Find ceiling and cloud cover
    cloud_cover = None
    for part in parts:
        if part.startswith(('FEW', 'SCT', 'BKN', 'OVC', 'OVX', 'VV')):
            cloud_cover = part[:3]  # Get first 3 chars
            try:
                ceiling_ft = int(part[3:6]) * 100
                break
            except (ValueError, IndexError):
                continue
        elif part in ['NSC', 'CLR', 'SKC', 'CAVOK']:
            ceiling_ft = 9999
    
    # Determine color code
    if ceiling_ft < 500 or visibility_miles < 1:
        color = '{68}'  # LIFR
    elif ceiling_ft < 1000 or visibility_miles < 3:
        color = '{63}'  # IFR
    elif ceiling_ft <= 3000 or visibility_miles <= 5:
        color = '{67}'  # MVFR
    else:
        color = '{66}'  # VFR

    white = '{69}'  # White color code

    # Generate the appropriate string based on cloud cover
    if cloud_cover:
        match cloud_cover:
            case "FEW":
                code = white*3 + color
                return code
            case "SCT":
                code = white*2 + color*2
                return code
            case "BKN":
                code = white + color*3
                return code
            case "OVC" | "OVX" | "VV":
                return color * 4
    return white*3 + color

def send_to_vesta():
    # METAR Vestaboard layout
    # MET VFR0000 MIL0 JT0000
    # METAR DUMP

    # TAF Vestaboard layout
    # 0000(JT) TAF DUMP

    # Format the data for Vestaboard
    if (settings.weather_type == "TAF"):
        formatted = f"{get_julia_time()} {api_data.replace('\n', '')}"
    else:
        formatted = f"MET VFR{get_vfr_color_code()} MIL{parse_mil_color()}JT{get_julia_time()}\\n{api_data.replace('\n', '')}"

    template = f'{{"components":[{{"template": "{formatted}"}}]}}'

    # Send to Vestaboard
    rw_client = vesta.ReadWriteClient(settings.api_key) # In .secrets.toml file
    rw_client.write_message(template)
    return f"Success: {template}"

def main_loop():
    logging.info(f"Starting Metarboard loop ({settings.interval}-minute intervals)")
    while True:
        try:            
            # Execute the function
            result = send_to_vesta()
            logging.info(f"Metarboard update successful: {result}")

            time.sleep(settings.interval * 60)
            
        except Exception as e:
            logging.error(f"Metarboard update failed: {str(e)}", exc_info=True)
            time.sleep(60)  # Wait before retrying

if __name__ == "__main__":
    main_loop()