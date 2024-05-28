# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class Event:
    @property
    def kind(self):
        """ """
        pass

    @property
    def position(self):
        """ """
        pass

    @property
    def timestamp(self):
        """ """
        pass

class EventKind:
    pass

class EventReceiver:
    def poll(self):
        """ """
        pass

class EventVec:
    def key_pressed(self, key):
        """
        Convinience method to check if a key was pressed in the event vector.
        """
        pass

    def key_released(self, key):
        """
        Convinience method to check if a key was released in the event vector.
        """
        pass
