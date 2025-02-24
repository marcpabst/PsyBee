# Creating your first experiment

This section will guide you through the process of creating your first PsyBee experiment. We will use the `pixi` package manager, but you can also use `pip`, `conda`, or `git` to install the library.

# Step 0: Install pixi

At the time of writing, you can install `pixi` using the following command:

=== ":fontawesome-brands-windows: Windows"

    ```powershell
    winget install prefix-dev.pixi
    ```

=== ":simple-apple: MacOS / :simple-linux: Linux"

    ```bash
    curl -fsSL https://pixi.sh/install.sh | bash
    ```


## Step 1: Create a new project and install PsyBee

First, navigate to the directory where you want to create your experiment and run the following command (you can replace `my_experiment` with the name of your experiment):

```bash
pixi init my_experiment
```

This will create a new directory called `my_experiment` with a `pixi.toml`. Then, we need to install the `PsyBee` library. We can do this using the `pixi` package manager:

```bash
pixi add python==3.12.* # or any other compatible version
pixi add psybee
```

## Step 2: Write the experiment

Create a new Python file called `first_experiment.py` in the `my_experiment` directory. This file will contain the code for your experiment.


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


## Step 3: Run the experiment

To run the experiment, we just need to execute the main.py file:

```bash
pixi run python first_experiment.py
```
