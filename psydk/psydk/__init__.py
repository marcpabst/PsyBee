# import the contents of the Rust library into the Python extension
print("importing psydk")
from .psydk import *
from .psydk import __all__

# optional: include the documentation from the Rust module
from .psydk import __doc__  # noqa: F401

# set gstreamer plugin environment variable to site-packages/psydk/.dylibs/
import platform

if platform.system() == 'Darwin':
    import os
    import sys
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".dylibs")
    os.environ["GST_PLUGIN_PATH"] = path + ":" + os.environ.get("GST_PLUGIN_PATH", "")
