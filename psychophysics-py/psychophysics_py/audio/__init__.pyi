# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class AudioDevice:
    pass

class AudioStimulus:
    """
    An AudioStimulus.
    """

    def pause(self):
        """ """
        pass

    def play(self):
        """ """
        pass

    def reset(self):
        """ """
        pass

    def restart(self):
        """ """
        pass

    def seek(self, time):
        """ """
        pass

    def set_volume(self, volume):
        """ """
        pass

class FileStimulus(AudioStimulus):
    """
    An audio stimulus that plays a sound from a file. See the `rodio` crate for
    supported file formats.

    Parameters
    ----------
    audio_device : AudioDevice
      The audio device that the stimulus will be played on.
    file_path : str
     The path to the audio file that will be played.
    """

    def pause(self):
        """ """
        pass

    def play(self):
        """ """
        pass

    def reset(self):
        """ """
        pass

    def restart(self):
        """ """
        pass

    def seek(self, time):
        """ """
        pass

    def set_volume(self, volume):
        """ """
        pass

class SineWaveStimulus(AudioStimulus):
    """
    A SineWaveStimulus.

    Parameters
    ----------
    audio_device : AudioDevice
       The audio device that the stimulus will be played on.
    frequency : float
      The frequency of the sine wave in Hz.
    duration : float
     The duration of the stimulus in seconds.
    """

    def pause(self):
        """ """
        pass

    def play(self):
        """ """
        pass

    def reset(self):
        """ """
        pass

    def restart(self):
        """ """
        pass

    def seek(self, time):
        """ """
        pass

    def set_volume(self, volume):
        """ """
        pass
