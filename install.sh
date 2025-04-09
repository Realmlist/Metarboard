#!/bin/bash

########################################################
#                                                      #
# !!EDIT THE SERVICE FILE BEFORE YOU RUN THIS SCRIPT!! #
#                                                      #
########################################################

# Create the secrets file for api key.
touch .secrets.toml
echo 'api_key = "READ_WRITE_API_KEY"' > .secrets.toml

# Create metarboard user
sudo adduser --system --no-create-home metarboard

# Copy the metarboard service file to the systemd directory
cp metarboard.service /etc/systemd/system/metarboard.service

# Reload systemd to recognize the new service
sudo systemctl daemon-reload

# Start the service
sudo systemctl start metarboard

# Enable auto-start on boot
sudo systemctl enable metarboard



####
# Live logs
# journalctl -u metarboard -f