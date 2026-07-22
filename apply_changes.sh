#!/usr/bin/env bash
#
# apply_changes.sh
#
# Make a backup of the destination haproxy.cfg and then copy the updated
# one, making sure the ownership is root:root and permission is 644
#

set -euo pipefail

sudo cp /etc/haproxy/haproxy.cfg{,.bak}
sudo mv ./haproxy.cfg /etc/haproxy/haproxy.cfg

sudo chown root:root /etc/haproxy/haproxy.cfg
sudo chmod 644 /etc/haproxy/haproxy.cfg

sudo haproxy -c -f /etc/haproxy/haproxy.cfg && sudo systemctl reload haproxy
