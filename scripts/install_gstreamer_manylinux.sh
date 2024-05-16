#!/bin/bash
# remove pkgconf
yum remove -y pkgconf
# install the real pkg-config
yum install -y pkgconfig
yum install -y alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-tools gstreamer1-plugins-base-devel

# print the version of gstreamer
gst-inspect-1.0 --version

# see if streamer-1.0.pc is in the pkg-config path
pkg-config --cflags --libs gstreamer-1.0

# run ldconfig
ldconfig

# list /usr/local/lib/pkgconfig
ls /usr/local/lib/pkgconfig

# print PKG_CONFIG_PATH
echo $PKG_CONFIG_PATH