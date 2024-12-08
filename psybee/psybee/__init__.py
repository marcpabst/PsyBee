from .psybee import *

# set gstreamer plugin environment variable to site-packages/psybee/.dylibs/
import os
import sys
import platform

if platform.system() == 'Darwin':
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".dylibs")
    os.environ["GST_PLUGIN_PATH"] = path + ":" + os.environ.get("GST_PLUGIN_PATH", "")

__doc__ = psybee.__doc__
if hasattr(psybee, "__all__"):
    __all__ = psybee.__all__
