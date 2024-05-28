# Your first experiment

This section will guide you through the process of creating your first PsyBee experiment using the `PsyBee` library. We will create a simple experiment that presents a white screen for 1 second, followed by a black screen for 1 second. This will be repeated 10 times.

## Step 1: Install the library

Please refer to the [installation guide](installation.md) for instructions on how to install the `PsyBee` library.

## Step 2: Create a new Python file

Create a new Python file called `first_experiment.py` and open it in your favourite text editor.

## Step 3: Write the experiment

First, we need to import the `PsyBee` library:

```python
import PsyBee as psy
```

Next, we need to create the experiment function. This function will create a window, present a white screen for 1 second, present a black screen for 1 second, and then close the window. This will be repeated 10 times.

```python
def my_experiment(wm):

    # create a window with default settings
    window = wm.create_default_window()

```