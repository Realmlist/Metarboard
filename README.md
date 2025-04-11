# Metarboard
Tool that grabs METAR/TAF information, formats it, and displays it on a Vestaboard.

I'm using this project to learn Python, don't expect professional-grade programming.


## Installation & usage
Run the pre-built docker container [`realmlist/metarboard`](https://hub.docker.com/r/realmlist/metarboard) or build it yourself.

### Docker compose:
```YAML
services:
  metarboard:
    image: realmlist/metarboard:latest
    restart: unless-stopped
    ports:
      - "5000:5000"
    volumes:
      - .:/app
    environment:
      - FLASK_ENV=production
    command: python app.py
```