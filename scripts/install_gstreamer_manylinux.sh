#!/bin/bash
yum install -y alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-tools
pkg-config --cflags --libs gstreamer-1.0