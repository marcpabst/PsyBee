#!/bin/bash
yum install -y alsa-lib-devel gstreamer1-devel gstreamer1-plugins-base-tools gstreamer1-plugins-base-devel gstreamer1-plugins-good gstreamer1-plugins-good-extras gstreamer1-plugins-ugly gstreamer1-plugins-bad-free gstreamer1-plugins-bad-free-devel gstreamer1-plugins-bad-free-extras
pkg-config --cflags --libs gstreamer-1.0