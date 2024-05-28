# Generated content DO NOT EDIT
from typing import Any, Callable, Dict, List, Optional, Tuple, Union, Sequence
from os import PathLike

class ExperimentManager:
    def create_default_window(self):
        """ """
        pass

    def prompt(self, prompt):
        """ """
        pass

class MainLoop:
    def get_available_monitors(self):
        """ """
        pass

    def prompt(self, prompt):
        """
        Prompt the user for input. This function will block the current thread
        until the user has entered a response.
        """
        pass

    def run_experiment(self, experiment_fn):
        """
        Runs your experiment function. This function will block the current thread
        until the experiment function returns.
        returns.

        Parameters
        ----------
        experiment_fn : callable
           The function that runs your experiment. This function should take a single argument, an instance of `ExperimentManager`, and should not return nothing.
        """
        pass

class Monitor:
    pass
