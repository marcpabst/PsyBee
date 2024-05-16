#!/bin/bash

dnf install -y python3-pip alsa-lib-devel gstreamer1 gstreamer1-devel

# print python version
python3 --version

# print pip version
pip3 --version

# list /usr/local/lib/pkgconfig
echo "Contents of /usr/local/lib/pkgconfig"
ls /usr/local/lib/pkgconfig

# list /usr/lib64/pkgconfig
echo "Contents of /usr/lib64/pkgconfig"
ls /usr/lib64/pkgconfig

# list /usr/lib/pkgconfig
echo "Contents of /usr/lib/pkgconfig"
ls /usr/lib/pkgconfig

# print PKG_CONFIG_PATH
echo $PKG_CONFIG_PATH