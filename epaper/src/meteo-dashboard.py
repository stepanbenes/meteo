#!/usr/bin/python
# -*- coding:utf-8 -*-
import sys
import os
import signal
import threading
import logging
import datetime
import time
from flask import Flask, request, jsonify
from PIL import Image, ImageDraw, ImageFont

picdir = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'pic')
libdir = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'lib')
if os.path.exists(libdir):
    sys.path.append(libdir)

from waveshare_epd import epd7in5_V2

logging.basicConfig(level=logging.DEBUG)

app = Flask(__name__)
shutdown_requested = threading.Event()

# current temperature and humidity
current_temp = -273.15
current_humidity = 0

# fonts
font18 = None
font24 = None
font35 = None
font48 = None

# e-paper state
epd = None
image = None
draw = None
display_lock = threading.Lock()

# handle SIGINT (Ctrl+C)
def signal_handler(sig, frame):
    logging.info("Signal received, shutting down...")
    shutdown_requested.set()

# draw a simple temperature graph
def draw_temperature_graph(draw, x, y, width, height, temp_history):
    if not temp_history:
        return

    # draw outer box
    draw.rectangle((x, y, x + width, y + height), outline=0)

    min_temp = min(temp_history)
    max_temp = max(temp_history)
    temp_range = max_temp - min_temp if max_temp != min_temp else 1

    points = []
    for i, temp in enumerate(temp_history):
        px = x + (i * width // (len(temp_history) - 1))
        py = y + height - int(((temp - min_temp) / temp_range) * height)
        points.append((px, py))

    # draw lines between points
    for i in range(len(points) - 1):
        draw.line([points[i], points[i + 1]], fill=0, width=2)

    # draw horizontal grid lines
    for i in range(1, 4):
        grid_y = y + (i * height // 4)
        draw.line((x, grid_y, x + width, grid_y), fill=0)

    # label max and min
    draw.text((x, y - 20), f"Max: {max_temp}°C", font=font18, fill=0)
    draw.text((x, y + height + 5), f"Min: {min_temp}°C", font=font18, fill=0)

# draw moon phase symbol
def get_moon_phase():
    now = datetime.datetime.utcnow()
    diff = now - datetime.datetime(2001, 1, 1)
    days = diff.days + (diff.seconds / 86400)
    lunations = days / 29.53058867
    return lunations % 1

def draw_moon_phase(draw, x, y, size, phase):
    # draw full moon base
    draw.ellipse((x, y, x + size, y + size), fill=0)
    if phase < 0.5:
        # waxing: cover right side
        shadow_width = int(size * (1 - 2 * phase))
        draw.ellipse((x + shadow_width, y, x + size + shadow_width, y + size), fill=255)
    elif phase > 0.5:
        # waning: cover left side
        shadow_width = int(size * (2 * (phase - 0.5)))
        draw.ellipse((x - shadow_width, y, x + size - shadow_width, y + size), fill=255)

# draw entire display
def update_display(temp, humidity, temp_history):
    global image, draw

    with display_lock:
        logging.info(f"Updating display with temp={temp}°C, humidity={humidity}%")

        # clear
        draw.rectangle((0, 0, epd.width, epd.height), fill=255)

        # current time
        current_time = datetime.datetime.now()

        # title
        draw.text((20, 20), "Meteo Station", font=font35, fill=0)

        # temperature & humidity
        draw.text((50, 80), f"{temp}°C", font=font48, fill=0)
        draw.text((50, 140), f"Humidity: {humidity}%", font=font24, fill=0)

        # clock & date
        draw.text((epd.width - 120, 20), current_time.strftime("%H:%M"), font=font24, fill=0)
        draw.text((epd.width - 120, 45), current_time.strftime("%Y-%m-%d"), font=font18, fill=0)

        # graph
        draw_temperature_graph(draw, 50, 200, 400, 150, temp_history)

        # moon
        draw.text((epd.width - 120, 75), "Moon:", font=font18, fill=0)
        draw_moon_phase(draw, epd.width - 80, 100, 40, get_moon_phase())

        # info panel
        info_x, info_y = 500, 200
        draw.rectangle((info_x, info_y, info_x + 200, info_y + 150), outline=0)
        draw.text((info_x + 10, info_y + 10), "Status", font=font24, fill=0)
        draw.text((info_x + 10, info_y + 40), f"Current: {temp}°C", font=font18, fill=0)
        draw.text((info_x + 10, info_y + 60), f"Humidity: {humidity}%", font=font18, fill=0)

        condition = "Warm" if temp > 25 else "Cool" if temp < 15 else "Moderate"
        draw.text((info_x + 10, info_y + 80), f"Condition: {condition}", font=font18, fill=0)

        draw.text((info_x + 10, info_y + 100), f"Updated: {current_time.strftime('%H:%M')}", font=font18, fill=0)

        # push to display
        epd.display(epd.getbuffer(image))

# HTTP status API
@app.route('/status', methods=['GET'])
def status():
    return jsonify({
        'temperature': current_temp,
        'humidity': current_humidity,
        'history': temp_history
    })

# HTTP update API
@app.route('/update', methods=['POST'])
def update():
    global current_temp, current_humidity
    data = request.json
    if not data:
        return jsonify({'error': 'Missing JSON data'}), 400
    try:
        temp = float(data.get('temperature'))
        humidity = int(data.get('humidity'))
        history = data['history']
        if not isinstance(history, list) or not all(isinstance(t, (int, float)) for t in history):
            return jsonify({'error': 'History must be a list of numbers'}), 400
        temp_history = [round(float(t), 1) for t in history]
        logging.info(f"Received temperature history: {temp_history}")
        if not (0 <= humidity <= 100):
            raise ValueError("Humidity must be between 0 and 100")
    except (TypeError, ValueError) as e:
        return jsonify({'error': f'Invalid data: {e}'}), 400

    current_temp = temp
    current_humidity = humidity
    update_display(current_temp, current_humidity, temp_history)
    return jsonify({'message': 'Display updated', 'temperature': current_temp, 'humidity': current_humidity, 'history': temp_history})

# HTTP history update API
@app.route('/history', methods=['POST'])
def update_history():
    global temp_history
    data = request.json
    if not data or 'history' not in data:
        return jsonify({'error': 'Missing "history" list'}), 400
    history = data['history']
    if not isinstance(history, list) or not all(isinstance(t, (int, float)) for t in history):
        return jsonify({'error': 'History must be a list of numbers'}), 400
    temp_history = [round(float(t), 1) for t in history]
    logging.info(f"Received temperature history: {temp_history}")
    update_display(current_temp, current_humidity, temp_history)
    return jsonify({'message': 'Temperature history updated', 'history': temp_history})

# blank & sleep display
def shutdown_display():
    global epd
    try:
        logging.info("Shutting down display...")
        epd.init()
        epd.Clear()
        epd.sleep()
    except Exception as e:
        logging.warning(f"Failed to sleep display: {e}")
    finally:
        epd7in5_V2.epdconfig.module_exit(cleanup=True)
        logging.info("GPIO and SPI cleaned up.")

# run Flask in a thread
def flask_thread():
    app.run(host='0.0.0.0', port=5000, threaded=True, use_reloader=False)

# entrypoint
def main():
    global epd, image, draw, font18, font24, font35, font48, temp_history

    signal.signal(signal.SIGINT, signal_handler)
    #signal.signal(signal.SIGTERM, signal_handler)

    logging.info("Starting E-Paper Display Web Service")

    # init display
    epd = epd7in5_V2.EPD()
    epd.init()
    epd.Clear()

    # load fonts
    font18 = ImageFont.truetype(os.path.join(picdir, 'Font.ttc'), 18)
    font24 = ImageFont.truetype(os.path.join(picdir, 'Font.ttc'), 24)
    font35 = ImageFont.truetype(os.path.join(picdir, 'Font.ttc'), 35)
    font48 = ImageFont.truetype(os.path.join(picdir, 'Font.ttc'), 48)

    # create canvas
    image = Image.new('1', (epd.width, epd.height), 255)
    draw = ImageDraw.Draw(image)

    # init temp history
    temp_history = []

    # draw initial screen
    update_display(current_temp, current_humidity, temp_history)

    # start web server
    t = threading.Thread(target=flask_thread)
    t.start()

    # wait for shutdown signal
    while not shutdown_requested.is_set():
        time.sleep(1)

    # clean up
    shutdown_display()
    logging.info("Shutdown complete.")
    logging.info("Forcing exit...")
    os.kill(os.getpid(), signal.SIGTERM)

if __name__ == "__main__":
    main()
