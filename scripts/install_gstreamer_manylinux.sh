#!/bin/bash
yum install -y alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-tools gstreamer1-plugins-base-devel

# print the version of gstreamer
gst-inspect-1.0 --version

# see if streamer-1.0.pc is in the pkg-config path
pkg-config --cflags --libs gstreamer-1.0