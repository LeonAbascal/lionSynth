# lionSynth in Rust
Work in progress

### Dependencies:
* Linux: [ALSA](http://www.escomposlinux.org/lfs-es/blfs-es-5.1/multimedia/alsa-tools.html)
  * **Ubuntu** `sudo apt install alsa_tools`

## Debug options
Here you will find some debug option that can be used to display useful information. As they
slow down the performance, they are deactivated by default, tho you can re-enable them in the
`Cargo.toml` file, adding them to the default feature list as follows:

```
[features]
default = ["verbose_modules"]   # Adding the option to the defaults of the program
verbose_modules = []            # Verbose modules option
```
You can find more information on the 
[documentation](https://doc.rust-lang.org/cargo/reference/features.html).

### Verbose modules
Makes the modules output their value on each iteration. I only recommend using this option
only on early debugging with a small set of samples generated, as output can get huge.

`cargo run --features verbose_modules`
