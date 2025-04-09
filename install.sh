#!/bin/bash

########################################################
#                                                      #
# !DOUBLE CHECK EVERYTHING BEFORE YOU RUN THIS SCRIPT! #
#                                                      #
########################################################

# Create and copy folder to /opt/realmlist/metarboard
sudo mkdir -p /opt/realmlist/metarboard
cp -r . /opt/realmlist/metarboard

# Create metarboard user
sudo adduser --system --no-create-home metarboard

# Change ownership of the metarboard folder to the metarboard user
sudo chown -R metarboard:metarboard /opt/realmlist/metarboard
sudo chmod -R 755 /opt/realmlist/metarboard  # For executables

# Create the secrets file for api key.
touch .secrets.toml
echo 'api_key = "READ_WRITE_API_KEY"' > .secrets.toml

# Copy the metarboard service file to the systemd directory
cp metarboard.service /etc/systemd/system/metarboard.service

# Create rsyslog config
sudo bash -c 'cat > /etc/rsyslog.d/metarboard.conf << EOF
# Metarboard Logging
if \$programname == '\''metarboard'\'' then /var/log/metarboard.log
& stop
EOF'

# Create log file with correct permissions
sudo touch /var/log/metarboard.log
sudo chown syslog:syslog /var/log/metarboard.log
sudo chmod 640 /var/log/metarboard.log

# Add logrotate configuration
sudo bash -c 'cat > /etc/logrotate.d/metarboard << EOF
/var/log/metarboard.log {
    weekly
    missingok
    rotate 4
    compress
    delaycompress
    notifempty
    create 640 syslog syslog
}
EOF'

# Restart rsyslog to ensure logs are captured
sudo systemctl restart rsyslog
sudo systemctl restart logrotate

# Reload systemd to recognize the new service
sudo systemctl daemon-reload

# Start the service
sudo systemctl start metarboard

# Enable auto-start on boot
sudo systemctl enable metarboard

####
# Check live logs
# journalctl -u metarboard -f