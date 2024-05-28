# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class Frame:
    """
    A Frame that can be used to render stimuli.
    """

    def set_bg_color(self, color):
        """
        Set the background color of the frame.
        """
        pass

    @property
    def stimuli(self):
        """
        The stimuli that are currently attached to the frame.
        """
        pass

class Window:
    def add_event_handler(self, kind, callback):
        """
        Add an event handler to the window. The event handler will be called
        whenever an event of the specified kind occurs.

        Parameters
        ----------
        kind : EventKind
          The kind of event to listen for.
        callback : callable
         The callback that will be called when the event occurs. The callback should take a single argument, an instance of `Event`.
        """
        pass

    def close(self):
        """ """
        pass

    def create_event_receiver(self):
        """
        Create a new EventReceiver for the window. The EventReceiver can be used
        to poll for events that have occurred on the window.

        Returns
        -------
        receiver : EventReceiver
            The EventReceiver that was created.
        """
        pass

    @property
    def cursor_visible(self):
        """ """
        pass

    def get_frame(self):
        """
        Obtain the next frame for the window.

        Returns
        -------
        frame : Frame
           The frame that was obtained from the window.
        """
        pass

    @property
    def stimuli(self):
        """
        Stimuli that are currently attached to the window.
        """
        pass

    def submit_frame(self, frame):
        """
        Submit a frame to the window. Might or might not block, depending
        on the current state of the underlying GPU queue.

        Parameters
        ----------
        frame : Frame
          The frame to submit to the window.
        """
        pass

class WindowOptions:
    pass
