# Getting started

> This guide contains code examples in both Rust and Python. You can switch between the two languages by using the tabs at the top of the code blocks.

## Installation

Psychophysics can be added as a dependency by using your favorite package manager like `cargo` for Rust or `pip`/`conda` for Python. 

```rust
cargo add psychophysics
```
```python
conda install psychophysics
```

See the [installation guide](installation.md) for more information and for instructions on how to install Psychophysics from source.

## Your first experiment

We use the term "experiment" to refer to a self-contained program that presents some sort stimuli to a participant and records their responses. In Psychophysics, an experiment is represented by an "experiment function" that takes an `ExperimentManager` as an argument. The `ExperimentManager` provides access to monitors, input devices, and other resources that are necessary for running an experiment.

This design ensures that your experiment is portable and can be run on different platforms without major modification. It also makes it possible to use multithreading which is important for running experiments with precise timing and on the web.

Here is a simple example of an experiment function that presents a fixation cross for 1 second and then waits for a key press:

```rust
use psychophysics::prelude::*;

#[psychophysics::experiment]
fn my_experiment(experiment: &ExperimentManager) -> Result<(), PsychophysicsError> {
    // Create a window
    let window = experiment.create_default_window()?;

    // Create a fixation cross
    let cross = FixationCrossStimulus::new(&window, color::BLACK, 0.1);

    // Present the fixation cross
    for frame in window {
        frame.add_stimulus(&cross);
    }
}
```
```python
import psychophysics as psy

@psy.experiment
def my_experiment(experiment):
    # Create a window
    window = experiment.create_default_window()

    # Create a fixation cross
    cross = psy.FixationCrossStimulus(window, psy.Color.BLACK, 0.1)

    # Present the fixation cross
    for frame in window:
        frame.add_stimulus(cross)
```

As you can see, it is usually not neccecary to create an `ExperimentManager` manually. Instead, you can use the `#[psychophysics::experiment]` (Rust) or the `@psy.experiment` decorator (Python) to let the library generate the `main` function or `__main__` code for you. Note that only one experiment function can be defined per project.