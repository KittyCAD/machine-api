# Machine API

This API intends to provide a standardized local interface to any machines used for manufacturing physical things.

## Getting started

### Config file

Here is a sample config file:

```toml
[machines.mk3]
type = "Usb"
baud = 115200
variant = "PrusaMk3"
slicer.type = "Prusa"
slicer.config = "config/prusa/mk3.ini"

[machines.nada]
type = "Noop"
nozzle_diameter = 0.6
state.state = "idle"
progress = 10.0
[[machines.nada.filaments]]
material.type = "pla"

[machines.neptune"]
type = "Moonraker"
endpoint = "http://192.168.1.102"
variant = "Neptune4"
slicer.type = "Prusa"
slicer.config = "config/prusa/neptune4.ini"
```

The cli looks by default for a file called `machine-api.toml` in the current
directory. You can also specify a different file with the `--config` flag.


### Running the server 

You can run this server locally assuming rust is installed:

```
cargo run -- serve --bind 0.0.0.0:8585
```

The full API is described by the OpenAPI spec, but to start you can list the connected machines:

```bash
$ curl http://localhost:8585/machines
```

The ID is what you'll use to identify the machine. You can use that ID to start a print job. 

For example, providing both an STL as `file`, and `params` as a json object with `machine_id` the same as above:

```bash
curl -X POST -F file=@input.stl -F 'params={"machine_id": "CZPX2418X004XK68718", "job_name": "my-cool-job"}' http://localhost:8585/print
```

Note: you may need to allow user permissions to USB devices. Alternatively, you can just run the server as root.

### CLI

You can also use machine-api as a CLI. `cargo run` with no parameters will give the available options.

## Contributing

### Regenerating the OpenAPI definition file

```bash
EXPECTORATE=overwrite cargo test --all openapi
```
