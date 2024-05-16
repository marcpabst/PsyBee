#!/bin/bash
# uninstall old python versions
dnf install -y alsa-lib-devel gstreamer1 gstreamer1-devel gstreamer1-plugins-base-devel

# install pip for python3
python3 -m ensurepip


# print python version
echo "Python version"
python3 --version

# print pip version
echo "Pip version"
pip3 --version

# list /usr/local/lib/pkgconfig
echo "Contents of /usr/local/lib/pkgconfig"
ls /usr/local/lib/pkgconfig

# list /usr/lib64/pkgconfig
echo "Contents of /usr/lib64/pkgconfig"
ls /usr/lib64/pkgconfig

# print PKG_CONFIG_PATH
echo "PKG_CONFIG_PATH"
echo $PKG_CONFIG_PATH