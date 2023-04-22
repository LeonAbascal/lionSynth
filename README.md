# lionSynth
Development of a Modular Digital Synthesizer Framework in Rust.

## University project description
Project divided into incremental phases to develop a framework for creating digital audio synthesizers for musical purposes. The architecture of the synthesizer would be composed of modules capable of reading or generating a signal, processing it and finally making it available to the next module in the chain, up to the OS output module. 

The features of each module are different depending on their purpose. Although their functionality is expected to be user-defined (programming), some basic modules (such as oscillators or gain stagers) will be included as part of the suite. 

Some additional features for the modules would be adjustable parameters that allow to change the behavior of the module; auxiliary inputs that interpret an output as a modifier of a parameter, happening automatically (sequencer) or periodically (LFO); or modules containing information about how they should be displayed on the screen. 

The behavior (and resulting sound) of the synthesizer will be drastically different depending on the modules chosen, their parameters and their arrangement. Therefore, a key feature will be to give the user the ability to define that layout. The most ambitious goals of this project would be to achieve the coordination of every component to accomplish real time output and the implementation of a graphical user interface flexible enough to adapt to a variable number of modules.

## Dependencies:
* Linux: [ALSA](http://www.escomposlinux.org/lfs-es/blfs-es-5.1/multimedia/alsa-tools.html)
  * **Ubuntu**: `sudo apt install libasound2-dev`
  * **Other**: not tested
* Windows: running ok

## Debug options
Here you will find some debug option that can be used to display useful information. As they
slow down the performance, they are deactivated by default, tho you can re-enable them in the
`Cargo.toml` file, adding them to the default feature list as follows:

```
[features]
default = ["verbose_modules"]   # Adding the option to the defaults of the program
verbose_modules = []            # Verbose modules option
```
You can find more information about features on the official
[documentation](https://doc.rust-lang.org/cargo/reference/features.html).

### Verbose modules
Makes small outputs of information that may be handy for debugging. Activated by default.

`cargo run --features verbose_modules`

## Testing
Some logging has been added to the tests. To display (although not as beautifully as it could,
thanks Rust) we can use the following option when running the tests:

`cargo test -- --nocapture`
